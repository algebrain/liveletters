use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

use native_tls::{TlsConnector, TlsStream};

use crate::{
    FetchBatch, ImapMailboxConfig, MailAuth, MailSecurity, MailboxCursor, ReceivedEmail,
    TransportError,
};

#[derive(Debug, Default)]
pub struct InMemoryImapMailbox {
    queued_emails: Vec<ReceivedEmail>,
}

impl InMemoryImapMailbox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_raw_email(&mut self, message_id: &str, raw_message: &str) {
        self.queued_emails.push(ReceivedEmail {
            message_id: message_id.to_owned(),
            raw_message: raw_message.to_owned(),
        });
    }

    pub fn fetch_new(&mut self) -> Result<Vec<ReceivedEmail>, TransportError> {
        Ok(self
            .fetch_batch(&MailboxCursor::start())?
            .into_emails())
    }

    pub fn fetch_batch(&mut self, cursor: &MailboxCursor) -> Result<FetchBatch, TransportError> {
        let start_index = cursor.last_seen_uid().unwrap_or(0) as usize;
        let emails = if start_index >= self.queued_emails.len() {
            Vec::new()
        } else {
            self.queued_emails[start_index..].to_vec()
        };
        let next_cursor = cursor.advance_to(self.queued_emails.len() as u64);

        Ok(FetchBatch::new(emails, next_cursor))
    }
}

#[derive(Debug, Clone)]
pub struct ConfiguredImapMailbox {
    config: ImapMailboxConfig,
}

impl ConfiguredImapMailbox {
    pub fn new(config: ImapMailboxConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ImapMailboxConfig {
        &self.config
    }

    pub fn fetch_new(&self, cursor: &MailboxCursor) -> Result<FetchBatch, TransportError> {
        let address = format!("{}:{}", self.config.server(), self.config.port());
        let stream = TcpStream::connect(&address)
            .map_err(|error| TransportError::Network(error.to_string()))?;
        let mut reader = match self.config.security() {
            MailSecurity::Tls => BufReader::new(ImapStream::Tls(connect_tls(stream, self.config.server())?)),
            MailSecurity::None | MailSecurity::StartTls => BufReader::new(ImapStream::Plain(stream)),
        };

        let greeting = read_line(&mut reader)?;
        if !greeting.starts_with("* OK") {
            return Err(TransportError::UnexpectedResponse(greeting.trim().to_owned()));
        }

        let command_offset = if self.config.security() == MailSecurity::StartTls {
            send_command(&mut reader, "a001", "STARTTLS")?;
            upgrade_imap_stream_to_tls(&mut reader, self.config.server())?;
            1
        } else {
            0
        };
        let login_tag = tag_at(command_offset);
        match self.config.auth() {
            MailAuth::None => {
                send_command(&mut reader, login_tag, "NOOP")?;
            }
            MailAuth::Password { username, password } => {
                send_command(
                    &mut reader,
                    login_tag,
                    &format!("LOGIN \"{}\" \"{}\"", escape_imap_string(username), escape_imap_string(password)),
                )?;
            }
        }

        send_command(
            &mut reader,
            tag_at(command_offset + 1),
            &format!("SELECT {}", self.config.mailbox()),
        )?;

        let start_uid = cursor.last_seen_uid().map(|uid| uid + 1).unwrap_or(1);
        let search_lines = send_command_collecting(
            &mut reader,
            tag_at(command_offset + 2),
            &format!("UID SEARCH UID {}:*", start_uid),
        )?;
        let uids = extract_search_uids(&search_lines);

        let mut emails = Vec::new();
        let mut next_cursor = cursor.clone();
        for uid in uids {
            let fetch_lines = send_command_collecting(
                &mut reader,
                tag_at(command_offset + 3),
                &format!("UID FETCH {uid} BODY.PEEK[]"),
            )?;
            let raw_message = extract_fetch_body(&fetch_lines)?;
            emails.push(ReceivedEmail {
                message_id: format!("imap-uid-{uid}"),
                raw_message,
            });
            next_cursor = next_cursor.advance_to(uid);
        }

        send_command(&mut reader, tag_at(command_offset + 4), "LOGOUT")?;

        Ok(FetchBatch::new(emails, next_cursor))
    }
}

enum ImapStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Read for ImapStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf),
            Self::Tls(stream) => stream.read(buf),
        }
    }
}

impl Write for ImapStream {
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

fn upgrade_imap_stream_to_tls(
    reader: &mut BufReader<ImapStream>,
    server_name: &str,
) -> Result<(), TransportError> {
    let plain_stream = match reader.get_mut() {
        ImapStream::Plain(stream) => stream.try_clone().map_err(|error| TransportError::Network(error.to_string()))?,
        ImapStream::Tls(_) => return Ok(()),
    };
    *reader.get_mut() = ImapStream::Tls(connect_tls(plain_stream, server_name)?);
    Ok(())
}

fn tag_at(offset: usize) -> &'static str {
    const TAGS: [&str; 6] = ["a001", "a002", "a003", "a004", "a005", "a006"];
    TAGS.get(offset).copied().unwrap_or("a999")
}

fn send_command(reader: &mut BufReader<ImapStream>, tag: &str, command: &str) -> Result<(), TransportError> {
    let response_lines = send_command_collecting(reader, tag, command)?;
    let status_line = response_lines
        .last()
        .ok_or_else(|| TransportError::UnexpectedResponse(String::new()))?;

    if status_line.starts_with(&format!("{tag} OK")) {
        Ok(())
    } else if status_line.starts_with(&format!("{tag} NO")) {
        Err(TransportError::AuthenticationFailed)
    } else {
        Err(TransportError::UnexpectedResponse(status_line.trim().to_owned()))
    }
}

fn send_command_collecting(
    reader: &mut BufReader<ImapStream>,
    tag: &str,
    command: &str,
) -> Result<Vec<String>, TransportError> {
    reader
        .get_mut()
        .write_all(format!("{tag} {command}\r\n").as_bytes())
        .map_err(|error| TransportError::Network(error.to_string()))?;
    reader
        .get_mut()
        .flush()
        .map_err(|error| TransportError::Network(error.to_string()))?;

    let mut lines = Vec::new();
    loop {
        let line = read_line(reader)?;
        let trimmed = line.trim_end_matches(['\r', '\n']).to_owned();
        let done = trimmed.starts_with(tag);
        lines.push(trimmed);
        if done {
            return Ok(lines);
        }
    }
}

fn read_line(reader: &mut BufReader<ImapStream>) -> Result<String, TransportError> {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| TransportError::Network(error.to_string()))?;
    Ok(line)
}

fn extract_search_uids(lines: &[String]) -> Vec<u64> {
    lines.iter()
        .filter_map(|line| line.strip_prefix("* SEARCH "))
        .flat_map(|tail| tail.split_whitespace())
        .filter_map(|uid| uid.parse::<u64>().ok())
        .collect()
}

fn extract_fetch_body(lines: &[String]) -> Result<String, TransportError> {
    let header_index = lines
        .iter()
        .position(|line| line.starts_with("* ") && line.contains("FETCH"))
        .ok_or_else(|| TransportError::UnexpectedResponse("missing FETCH header".to_owned()))?;
    let header = &lines[header_index];

    let Some(start) = header.rfind('{') else {
        return Err(TransportError::UnexpectedResponse(header.clone()));
    };
    let Some(end) = header.rfind('}') else {
        return Err(TransportError::UnexpectedResponse(header.clone()));
    };

    let literal_size = header[start + 1..end]
        .parse::<usize>()
        .map_err(|_| TransportError::UnexpectedResponse(header.clone()))?;

    let mut literal = String::new();
    for line in lines.iter().skip(header_index + 1) {
        if line == ")" || line.starts_with('a') {
            continue;
        }

        if !literal.is_empty() {
            literal.push('\n');
        }
        literal.push_str(line);

        if literal.len() >= literal_size {
            literal.truncate(literal_size);
            return Ok(literal);
        }
    }

    Err(TransportError::UnexpectedResponse("short FETCH literal".to_owned()))
}

fn escape_imap_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
