use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

use native_tls::{TlsConnector, TlsStream};

use crate::{
    FetchBatch, ImapMailboxConfig, MailAuth, MailSecurity, MailboxCursor, ReceivedEmail,
    TransportError,
};

const LIVELETTERS_PROTOCOL_HEADER: &str = "X-LiveLetters-Protocol";
const LIVELETTERS_PROTOCOL_VERSION: &str = "v1";

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
        liveletters_log::log_info(format!(
            "imap.connect host={} port={} security={}",
            self.config.server(),
            self.config.port(),
            self.config.security().as_str(),
        ));
        let stream = TcpStream::connect(&address).map_err(|error| {
            liveletters_log::log_error(format!("imap.connect error={error}"));
            TransportError::Network(error.to_string())
        })?;
        let mut reader = match self.config.security() {
            MailSecurity::Tls => {
                BufReader::new(ImapStream::Tls(connect_tls(stream, self.config.server())?))
            }
            MailSecurity::None | MailSecurity::StartTls => {
                BufReader::new(ImapStream::Plain(stream))
            }
        };

        let greeting = read_line(&mut reader)?;
        if !greeting.starts_with("* OK") {
            liveletters_log::log_error(format!("imap.greeting response={}", greeting.trim()));
            return Err(TransportError::UnexpectedResponse(
                greeting.trim().to_owned(),
            ));
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
                liveletters_log::log_info(format!("imap.login user={username}"));
                send_command(
                    &mut reader,
                    login_tag,
                    &format!(
                        "LOGIN \"{}\" \"{}\"",
                        escape_imap_string(username),
                        escape_imap_string(password)
                    ),
                )?;
            }
        }

        send_command(
            &mut reader,
            tag_at(command_offset + 1),
            &format!("SELECT {}", self.config.mailbox()),
        )?;
        liveletters_log::log_info(format!("imap.select mailbox={}", self.config.mailbox()));

        let start_uid = cursor.last_seen_uid().map(|uid| uid + 1).unwrap_or(1);
        let search_result = search_liveletters_uids(
            &mut reader,
            tag_at(command_offset + 2),
            tag_at(command_offset + 3),
            start_uid,
        )?;
        liveletters_log::log_info(format!(
            "imap.search.result start_uid={start_uid} matched={}",
            search_result.uids.len(),
        ));
        let uids = search_result.uids;

        let mut emails = Vec::new();
        let mut next_cursor = cursor.clone();
        for uid in uids {
            let raw_message = fetch_body_literal(&mut reader, tag_at(command_offset + 3), uid)?;
            emails.push(ReceivedEmail {
                message_id: format!("imap-uid-{uid}"),
                raw_message,
            });
            next_cursor = next_cursor.advance_to(uid);
        }
        if let Some(max_seen_uid) = search_result.max_seen_uid {
            next_cursor = next_cursor.advance_to(max_seen_uid);
        }

        send_command(&mut reader, tag_at(command_offset + 4), "LOGOUT")?;

        Ok(FetchBatch::new(emails, next_cursor))
    }
}

struct SearchResult {
    uids: Vec<u64>,
    max_seen_uid: Option<u64>,
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

fn connect_tls(
    stream: TcpStream,
    server_name: &str,
) -> Result<TlsStream<TcpStream>, TransportError> {
    let connector =
        TlsConnector::new().map_err(|error| TransportError::Network(error.to_string()))?;
    connector
        .connect(server_name, stream)
        .map_err(|error| TransportError::Network(error.to_string()))
}

fn upgrade_imap_stream_to_tls(
    reader: &mut BufReader<ImapStream>,
    server_name: &str,
) -> Result<(), TransportError> {
    let plain_stream = match reader.get_mut() {
        ImapStream::Plain(stream) => stream
            .try_clone()
            .map_err(|error| TransportError::Network(error.to_string()))?,
        ImapStream::Tls(_) => return Ok(()),
    };
    *reader.get_mut() = ImapStream::Tls(connect_tls(plain_stream, server_name)?);
    Ok(())
}

fn tag_at(offset: usize) -> &'static str {
    const TAGS: [&str; 6] = ["a001", "a002", "a003", "a004", "a005", "a006"];
    TAGS.get(offset).copied().unwrap_or("a999")
}

fn send_command(
    reader: &mut BufReader<ImapStream>,
    tag: &str,
    command: &str,
) -> Result<(), TransportError> {
    let response_lines = send_command_collecting(reader, tag, command)?;
    let status_line = response_lines
        .last()
        .ok_or_else(|| TransportError::UnexpectedResponse(String::new()))?;

    if status_line.starts_with(&format!("{tag} OK")) {
        Ok(())
    } else if status_line.starts_with(&format!("{tag} NO")) {
        Err(TransportError::AuthenticationFailed)
    } else {
        Err(TransportError::UnexpectedResponse(
            status_line.trim().to_owned(),
        ))
    }
}

fn command_status(lines: &[String], tag: &str) -> Result<CommandStatus, TransportError> {
    let status_line = lines
        .last()
        .ok_or_else(|| TransportError::UnexpectedResponse(String::new()))?;
    if status_line.starts_with(&format!("{tag} OK")) {
        Ok(CommandStatus::Ok)
    } else if status_line.starts_with(&format!("{tag} NO")) {
        Ok(CommandStatus::No)
    } else if status_line.starts_with(&format!("{tag} BAD")) {
        Ok(CommandStatus::Bad)
    } else {
        Err(TransportError::UnexpectedResponse(
            status_line.trim().to_owned(),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandStatus {
    Ok,
    No,
    Bad,
}

fn send_command_collecting(
    reader: &mut BufReader<ImapStream>,
    tag: &str,
    command: &str,
) -> Result<Vec<String>, TransportError> {
    write_command(reader, tag, command)?;

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

fn write_command(
    reader: &mut BufReader<ImapStream>,
    tag: &str,
    command: &str,
) -> Result<(), TransportError> {
    reader
        .get_mut()
        .write_all(format!("{tag} {command}\r\n").as_bytes())
        .map_err(|error| TransportError::Network(error.to_string()))?;
    reader
        .get_mut()
        .flush()
        .map_err(|error| TransportError::Network(error.to_string()))?;
    Ok(())
}

fn read_until_tag(reader: &mut BufReader<ImapStream>, tag: &str) -> Result<(), TransportError> {
    loop {
        let line = read_line(reader)?;
        let trimmed = line.trim_end_matches(['\r', '\n']).to_owned();
        if trimmed.starts_with(&format!("{tag} OK")) {
            return Ok(());
        }
        if trimmed.starts_with(&format!("{tag} NO")) {
            return Err(TransportError::AuthenticationFailed);
        }
        if trimmed.starts_with(tag) {
            return Err(TransportError::UnexpectedResponse(trimmed));
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
    lines
        .iter()
        .filter_map(|line| line.strip_prefix("* SEARCH "))
        .flat_map(|tail| tail.split_whitespace())
        .filter_map(|uid| uid.parse::<u64>().ok())
        .collect()
}

fn search_liveletters_uids(
    reader: &mut BufReader<ImapStream>,
    search_tag: &str,
    fetch_tag: &str,
    start_uid: u64,
) -> Result<SearchResult, TransportError> {
    let header_command = format!(
        "UID SEARCH UID {start_uid}:* HEADER {LIVELETTERS_PROTOCOL_HEADER} {LIVELETTERS_PROTOCOL_VERSION}"
    );
    let header_search_lines = send_command_collecting(reader, search_tag, &header_command)?;
    match command_status(&header_search_lines, search_tag)? {
        CommandStatus::Ok => {
            let matched = extract_search_uids(&header_search_lines);
            if matched.is_empty() {
                fallback_search_by_fetching_headers(
                    reader,
                    search_tag,
                    fetch_tag,
                    start_uid,
                    "primary search returned 0 candidates",
                )
            } else {
                Ok(SearchResult {
                    uids: matched,
                    max_seen_uid: None,
                })
            }
        }
        CommandStatus::No | CommandStatus::Bad => fallback_search_by_fetching_headers(
            reader,
            search_tag,
            fetch_tag,
            start_uid,
            header_search_lines
                .last()
                .map(String::as_str)
                .unwrap_or_default(),
        ),
    }
}

fn fallback_search_by_fetching_headers(
    reader: &mut BufReader<ImapStream>,
    search_tag: &str,
    fetch_tag: &str,
    start_uid: u64,
    unsupported_reason: &str,
) -> Result<SearchResult, TransportError> {
    let all_command = format!("UID SEARCH UID {start_uid}:*");
    let all_search_lines = send_command_collecting(reader, search_tag, &all_command)?;
    if command_status(&all_search_lines, search_tag)? != CommandStatus::Ok {
        return Err(TransportError::UnexpectedResponse(
            all_search_lines.last().cloned().unwrap_or_default(),
        ));
    }

    let candidates = extract_search_uids(&all_search_lines);
    let max_seen_uid = candidates.iter().copied().max();
    let mut liveletters_uids = Vec::new();
    for uid in candidates {
        let headers = fetch_header_literal(reader, fetch_tag, uid).map_err(|error| match error {
            TransportError::UnexpectedResponse(message) => TransportError::UnexpectedResponse(
                format!(
                    "server does not support LiveLetters header filtering after `{unsupported_reason}`: {message}"
                ),
            ),
            other => other,
        })?;
        if has_liveletters_protocol_header(&headers) {
            liveletters_uids.push(uid);
        }
    }

    Ok(SearchResult {
        uids: liveletters_uids,
        max_seen_uid,
    })
}

fn has_liveletters_protocol_header(headers: &str) -> bool {
    headers.lines().any(|line| {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };
        name.trim()
            .eq_ignore_ascii_case(LIVELETTERS_PROTOCOL_HEADER)
            && value
                .trim()
                .eq_ignore_ascii_case(LIVELETTERS_PROTOCOL_VERSION)
    })
}

fn fetch_header_literal(
    reader: &mut BufReader<ImapStream>,
    tag: &str,
    uid: u64,
) -> Result<String, TransportError> {
    fetch_literal(
        reader,
        tag,
        &format!("UID FETCH {uid} BODY.PEEK[HEADER.FIELDS ({LIVELETTERS_PROTOCOL_HEADER})]"),
    )
}

fn fetch_body_literal(
    reader: &mut BufReader<ImapStream>,
    tag: &str,
    uid: u64,
) -> Result<String, TransportError> {
    fetch_literal(reader, tag, &format!("UID FETCH {uid} BODY.PEEK[]"))
}

fn fetch_literal(
    reader: &mut BufReader<ImapStream>,
    tag: &str,
    command: &str,
) -> Result<String, TransportError> {
    write_command(reader, tag, command)?;
    let literal_size = loop {
        let line = read_line(reader)?;
        let trimmed = line.trim_end_matches(['\r', '\n']).to_owned();
        if trimmed.starts_with(tag) {
            return Err(TransportError::UnexpectedResponse(trimmed));
        }
        if trimmed.starts_with("* ") && trimmed.contains("FETCH") {
            break parse_literal_size(&trimmed)?;
        }
    };

    let mut bytes = vec![0_u8; literal_size];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| TransportError::Network(error.to_string()))?;
    read_until_tag(reader, tag)?;

    String::from_utf8(bytes).map_err(|error| {
        TransportError::UnexpectedResponse(format!("FETCH body is not UTF-8: {error}"))
    })
}

fn parse_literal_size(header: &str) -> Result<usize, TransportError> {
    let Some(start) = header.rfind('{') else {
        return Err(TransportError::UnexpectedResponse(header.to_owned()));
    };
    let Some(end) = header.rfind('}') else {
        return Err(TransportError::UnexpectedResponse(header.to_owned()));
    };
    header[start + 1..end]
        .parse::<usize>()
        .map_err(|_| TransportError::UnexpectedResponse(header.to_owned()))
}

fn escape_imap_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
