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
        let (mut reader, command_offset) = self.open_session()?;

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

    /// Возвращает минимальный UID писем, пришедших не старше `days`
    /// суток. Используется при первом запуске с `initial_lookback_days`
    /// и при backfill. Открывает отдельное IMAP-соединение, выполняет
    /// `UID SEARCH SINCE <since_date>`, закрывает соединение.
    pub fn find_min_uid_since_days(&self, days: u32) -> Result<u64, TransportError> {
        if days == 0 {
            return Ok(1);
        }
        let (mut reader, command_offset) = self.open_session()?;
        let since_date = since_date_for_today_minus(days);
        let since_tag = tag_at(command_offset + 2);
        let search_lines = send_command_collecting(
            &mut reader,
            since_tag,
            &format!("UID SEARCH SINCE {since_date}"),
        )?;
        if command_status(&search_lines, since_tag)? != CommandStatus::Ok {
            return Err(TransportError::UnexpectedResponse(
                search_lines.last().cloned().unwrap_or_default(),
            ));
        }
        let uids = extract_search_uids(&search_lines);
        let min_uid = uids.iter().copied().min().unwrap_or(1);
        send_command(&mut reader, tag_at(command_offset + 3), "LOGOUT")?;
        Ok(min_uid)
    }

    /// Курсор, который "видел" все UID < since_uid, где since_uid —
    /// минимальный UID писем за последние `days` суток. Используется
    /// при backfill. Возвращает курсор с `start_uid = max(1, min_uid)`.
    pub fn anchor_for_backfill(&self, days: u32) -> Result<MailboxCursor, TransportError> {
        let since_uid = self.find_min_uid_since_days(days)?;
        Ok(MailboxCursor::start_with_since_uid(since_uid.max(1)))
    }

    fn open_session(&self) -> Result<(BufReader<ImapStream>, usize), TransportError> {
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

        Ok((reader, command_offset))
    }
}

fn since_date_for_today_minus(days: u32) -> String {
    // Без внешних зависимостей: вычисляем дату `today - days` в формате
    // IMAP "DD-Mon-YYYY" (например, "09-Jun-2026") через стандартную
    // библиотеку. Поддерживаем максимум ~135 лет (i64 секунд), этого
    // хватит для любого разумного `days`.
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let secs_in_day: i64 = 86_400;
    let target = now - (days as i64) * secs_in_day;
    let days_since_epoch = target / secs_in_day;
    // Алгоритм: вычисляем год/месяц/день из дней с 1970-01-01.
    // Используем стандартный civil_from_days из Howard Hinnant.
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };
    let month_abbr = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!("{:02}-{}-{:04}", d, month_abbr[(m - 1) as usize], year)
}

#[cfg(test)]
mod tests {
    use super::since_date_for_today_minus;

    #[test]
    fn since_date_for_today_minus_zero_is_today() {
        // days=0 → today. Сейчас, как минимум, должно быть валидной
        // датой в формате "DD-Mon-YYYY".
        let s = since_date_for_today_minus(0);
        assert_eq!(s.len(), 11, "формат DD-Mon-YYYY = 11 символов");
    }

    #[test]
    fn since_date_for_today_minus_one_is_yesterday() {
        // days=1 → вчера. Проверяем, что строка валидна по формату.
        let s = since_date_for_today_minus(1);
        assert_eq!(s.len(), 11);
        let bytes = s.as_bytes();
        assert!(bytes[2] == b'-' && bytes[6] == b'-');
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
            let mut matched = extract_search_uids(&header_search_lines);
            matched.retain(|uid| *uid >= start_uid);
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

    let mut candidates = extract_search_uids(&all_search_lines);
    let max_seen_uid = candidates.iter().copied().max();
    candidates.retain(|uid| *uid >= start_uid);
    let mut liveletters_uids = Vec::new();
    for uid in candidates {
        // Шаг 2: BODY.PEEK[HEADER.FIELDS (...)]
        let headers_attempt = fetch_header_literal(reader, fetch_tag, uid);
        let has_header = match headers_attempt {
            Ok(headers) => has_liveletters_protocol_header(&headers),
            Err(TransportError::UnexpectedResponse(msg))
                if msg.contains("PARSE") || msg.contains("BAD") =>
            {
                // Шаг 3: BODY.PEEK[HEADER] (без .FIELDS)
                match fetch_full_headers_literal(reader, fetch_tag, uid) {
                    Ok(full_headers) => has_liveletters_protocol_header(&full_headers),
                    Err(TransportError::UnexpectedResponse(msg2))
                        if msg2.contains("PARSE") || msg2.contains("BAD") =>
                    {
                        // Шаг 4: BODY.PEEK[] (всё тело)
                        let body = fetch_body_literal(reader, fetch_tag, uid)?;
                        extract_liveletters_protocol_header_from_body(&body)
                    }
                    Err(other) => {
                        return Err(TransportError::UnexpectedResponse(format!(
                            "server does not support LiveLetters header filtering after `{unsupported_reason}` (uid {uid}): {other:?}"
                        )));
                    }
                }
            }
            Err(other) => {
                return Err(TransportError::UnexpectedResponse(format!(
                    "server does not support LiveLetters header filtering after `{unsupported_reason}`: {other:?}"
                )));
            }
        };
        if has_header {
            liveletters_uids.push(uid);
        }
    }

    Ok(SearchResult {
        uids: liveletters_uids,
        max_seen_uid,
    })
}

fn fetch_full_headers_literal(
    reader: &mut BufReader<ImapStream>,
    tag: &str,
    uid: u64,
) -> Result<String, TransportError> {
    fetch_literal(reader, tag, &format!("UID FETCH {uid} BODY.PEEK[HEADER]"))
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

fn extract_liveletters_protocol_header_from_body(body: &str) -> bool {
    // Простое сканирование первых ~200 строк письма на наличие
    // строки "X-LiveLetters-Protocol: v1". Полный MIME-парсинг
    // избыточен для этой задачи.
    body.lines().take(200).any(|line| {
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
