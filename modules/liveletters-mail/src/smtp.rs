use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

use native_tls::{TlsConnector, TlsStream};

use crate::{MailAuth, MailSecurity, OutgoingEmail, SendStatus, SmtpTransportConfig, TransportError};

#[derive(Debug, Default)]
pub struct InMemorySmtpTransport {
    sent_emails: Vec<OutgoingEmail>,
}

impl InMemorySmtpTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send(&mut self, email: OutgoingEmail) -> Result<SendStatus, TransportError> {
        self.sent_emails.push(email);
        Ok(SendStatus::Sent)
    }

    pub fn sent_emails(&self) -> &[OutgoingEmail] {
        &self.sent_emails
    }
}

#[derive(Debug, Clone)]
pub struct ConfiguredSmtpTransport {
    config: SmtpTransportConfig,
}

impl ConfiguredSmtpTransport {
    pub fn new(config: SmtpTransportConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &SmtpTransportConfig {
        &self.config
    }

    pub fn send(&self, email: &OutgoingEmail) -> Result<SendStatus, TransportError> {
        let address = format!("{}:{}", self.config.server(), self.config.port());
        let stream = TcpStream::connect(&address)
            .map_err(|error| TransportError::Network(error.to_string()))?;
        let mut reader = match self.config.security() {
            MailSecurity::Tls => BufReader::new(SmtpStream::Tls(connect_tls(stream, self.config.server())?)),
            MailSecurity::None | MailSecurity::StartTls => {
                BufReader::new(SmtpStream::Plain(stream))
            }
        };

        read_response(&mut reader, 220)?;
        send_command(&mut reader, &format!("EHLO {}\r\n", self.config.hello_domain()), 250)?;

        if self.config.security() == MailSecurity::StartTls {
            send_command(&mut reader, "STARTTLS\r\n", 220)?;
            upgrade_smtp_stream_to_tls(&mut reader, self.config.server())?;
            send_command(&mut reader, &format!("EHLO {}\r\n", self.config.hello_domain()), 250)?;
        }

        match self.config.auth() {
            MailAuth::None => {}
            MailAuth::Password { username, password } => {
                let token = base64_encode(&format!("\u{0}{username}\u{0}{password}"));
                send_command(&mut reader, &format!("AUTH PLAIN {token}\r\n"), 235)?;
            }
        }

        send_command(&mut reader, &format!("MAIL FROM:<{}>\r\n", email.from), 250)?;
        send_command(&mut reader, &format!("RCPT TO:<{}>\r\n", email.to), 250)?;
        send_command(&mut reader, "DATA\r\n", 354)?;

        let data = normalize_data_block(&email.raw_message);
        reader
            .get_mut()
            .write_all(data.as_bytes())
            .map_err(|error| TransportError::Network(error.to_string()))?;
        reader
            .get_mut()
            .write_all(b"\r\n.\r\n")
            .map_err(|error| TransportError::Network(error.to_string()))?;
        reader
            .get_mut()
            .flush()
            .map_err(|error| TransportError::Network(error.to_string()))?;

        read_response(&mut reader, 250)?;
        send_command(&mut reader, "QUIT\r\n", 221)?;

        Ok(SendStatus::Sent)
    }
}

enum SmtpStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Read for SmtpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf),
            Self::Tls(stream) => stream.read(buf),
        }
    }
}

impl Write for SmtpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buf),
            Self::Tls(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            Self::Tls(stream) => stream.flush(),
        }
    }
}

fn connect_tls(stream: TcpStream, server_name: &str) -> Result<TlsStream<TcpStream>, TransportError> {
    let connector = TlsConnector::new().map_err(|error| TransportError::Network(error.to_string()))?;
    connector
        .connect(server_name, stream)
        .map_err(|error| TransportError::Network(error.to_string()))
}

fn upgrade_smtp_stream_to_tls(
    reader: &mut BufReader<SmtpStream>,
    server_name: &str,
) -> Result<(), TransportError> {
    let plain_stream = match reader.get_mut() {
        SmtpStream::Plain(stream) => stream.try_clone().map_err(|error| TransportError::Network(error.to_string()))?,
        SmtpStream::Tls(_) => return Ok(()),
    };
    *reader.get_mut() = SmtpStream::Tls(connect_tls(plain_stream, server_name)?);
    Ok(())
}

fn send_command(
    reader: &mut BufReader<SmtpStream>,
    command: &str,
    expected_code: u16,
) -> Result<String, TransportError> {
    reader
        .get_mut()
        .write_all(command.as_bytes())
        .map_err(|error| TransportError::Network(error.to_string()))?;
    reader
        .get_mut()
        .flush()
        .map_err(|error| TransportError::Network(error.to_string()))?;
    read_response(reader, expected_code)
}

fn read_response(reader: &mut BufReader<SmtpStream>, expected_code: u16) -> Result<String, TransportError> {
    let mut response = String::new();
    loop {
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|error| TransportError::Network(error.to_string()))?;
        if bytes_read == 0 {
            return Err(TransportError::UnexpectedResponse(response));
        }

        response.push_str(&line);
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.len() < 4 {
            continue;
        }

        let Ok(code) = trimmed[0..3].parse::<u16>() else {
            continue;
        };

        if &trimmed[3..4] == "-" {
            continue;
        }

        if code != expected_code {
            return match code {
                535 => Err(TransportError::AuthenticationFailed),
                _ => Err(TransportError::UnexpectedResponse(trimmed.to_owned())),
            };
        }

        return Ok(response);
    }
}

fn normalize_data_block(raw_message: &str) -> String {
    raw_message
        .replace("\r\n", "\n")
        .lines()
        .map(|line| {
            if let Some(stripped) = line.strip_prefix('.') {
                format!("..{stripped}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n")
}

fn base64_encode(input: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let bytes = input.as_bytes();
    let mut output = String::new();
    let mut index = 0;

    while index < bytes.len() {
        let chunk = &bytes[index..usize::min(index + 3, bytes.len())];
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        let n0 = b0 >> 2;
        let n1 = ((b0 & 0b0000_0011) << 4) | (b1 >> 4);
        let n2 = ((b1 & 0b0000_1111) << 2) | (b2 >> 6);
        let n3 = b2 & 0b0011_1111;

        output.push(TABLE[n0 as usize] as char);
        output.push(TABLE[n1 as usize] as char);

        if chunk.len() > 1 {
            output.push(TABLE[n2 as usize] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(TABLE[n3 as usize] as char);
        } else {
            output.push('=');
        }

        index += 3;
    }

    output
}
