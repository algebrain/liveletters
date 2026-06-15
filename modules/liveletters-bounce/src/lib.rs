//! Распознавание и парсинг DSN-bounce (Delivery Status Notification) по RFC 3462.
//!
//! DSN-bounce — это автоматическое уведомление от почтового сервера о
//! недоставленном письме. В `lltt` используется для сопоставления отказа
//! с нашим исходящим `SubscriptionRequested` и перевода подписки из `pending`
//! в `failed`.
//!
//! Распознавание «наших» bounce:
//! - письмо должно иметь `Content-Type: multipart/report; report-type=delivery-status`;
//! - внутри `message/delivery-status` должно быть `Action: failed` (или другое);
//! - `Final-Recipient: rfc822; <addr>` — кому не доставлено;
//! - `Original-Message-ID` (если есть) — для сопоставления с нашим outbox.
//!
//! Не путать с ARF (Abuse Reporting Format, RFC 5965) — `report-type=feedback`.
//! И не искать «все письма от Mail Delivery Subsystem» — этот префикс
//! локализуется (Mail.ru, Яндекс, Outlook используют разные строки).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BounceReport {
    pub action: BounceAction,
    pub status: String,
    pub final_recipient: String,
    pub diagnostic_code: String,
    pub original_message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BounceAction {
    Failed,
    Delayed,
    Delivered,
    Relayed,
    Expanded,
    Other(String),
}

/// Распознать и распарсить DSN-bounce в сыром MIME-сообщении.
/// Возвращает `Ok(None)` если письмо не DSN-bounce.
pub fn parse_dsn(raw_email: &str) -> Result<Option<BounceReport>, BounceError> {
    // Поиск заголовка Content-Type с multipart/report; report-type=delivery-status
    let header_end = raw_email.find("\r\n\r\n").unwrap_or(raw_email.len());
    let headers = &raw_email[..header_end];

    let content_type = find_header(headers, "Content-Type")
        .ok_or_else(|| BounceError::MissingHeader("Content-Type".to_owned()))?;

    if !content_type.contains("multipart/report")
        || !content_type.contains("report-type=delivery-status")
    {
        return Ok(None);
    }

    // Парсим body — для простоты ищем вхождения ключевых полей
    let body_start = header_end + 4;
    let body = raw_email.get(body_start..).unwrap_or("");

    let action_str = find_field(body, "Action").unwrap_or_default();
    let action = parse_action(&action_str);
    let status = find_field(body, "Status").unwrap_or_default();
    let final_recipient_full = find_field(body, "Final-Recipient").unwrap_or_default();
    let final_recipient = final_recipient_full
        .split_once(';')
        .map(|(_, addr)| addr.trim().to_owned())
        .unwrap_or(final_recipient_full);
    let diagnostic_code_full = find_field(body, "Diagnostic-Code").unwrap_or_default();
    let diagnostic_code = diagnostic_code_full
        .split_once(';')
        .map(|(_, code)| code.trim().to_owned())
        .unwrap_or(diagnostic_code_full);
    let original_message_id = find_field(body, "Original-Message-ID")
        .or_else(|| find_message_id_in_rfc822(body))
        .map(|s| {
            s.trim()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_owned()
        });

    Ok(Some(BounceReport {
        action,
        status,
        final_recipient,
        diagnostic_code,
        original_message_id,
    }))
}

fn parse_action(s: &str) -> BounceAction {
    match s.to_ascii_lowercase().as_str() {
        "failed" => BounceAction::Failed,
        "delayed" => BounceAction::Delayed,
        "delivered" => BounceAction::Delivered,
        "relayed" => BounceAction::Relayed,
        "expanded" => BounceAction::Expanded,
        other => BounceAction::Other(other.to_owned()),
    }
}

fn find_header(headers: &str, name: &str) -> Option<String> {
    for line in headers.split("\r\n") {
        if let Some(rest) = line.strip_prefix(name).and_then(|s| s.strip_prefix(':')) {
            return Some(rest.trim().to_owned());
        }
    }
    None
}

/// Ищет в body поле вида `Name: value` (на начало строки). Поиск идёт
/// по всему телу, потому что `message/delivery-status` может быть
/// не первой секцией multipart/report.
fn find_field(body: &str, name: &str) -> Option<String> {
    let prefix_lower = format!("{}:", name).to_ascii_lowercase();
    for line in body.split("\r\n") {
        // Пропускаем MIME-границы и пустые строки
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        let line_lower = line.to_ascii_lowercase();
        if let Some(rest) = line_lower.strip_prefix(&prefix_lower) {
            let idx = line.len() - rest.len();
            return Some(line[idx..].trim().to_owned());
        }
    }
    None
}

/// Ищет `Message-ID: <...>` внутри `message/rfc822` секции (копия нашего исходящего).
fn find_message_id_in_rfc822(body: &str) -> Option<String> {
    let start = body.find("Content-Type: message/rfc822")?;
    let after = &body[start..];
    // Ищем границу MIME-секции
    let section_start = after.find("\r\n\r\n")?;
    let section = &after[section_start..];
    // Внутри может быть ещё одно вложение — но обычно Message-ID идёт в самом верху
    let id_line = section
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("message-id:"))?;
    Some(id_line["message-id:".len()..].trim().to_owned())
}

#[derive(Debug, thiserror::Error)]
pub enum BounceError {
    #[error("отсутствует обязательный заголовок: {0}")]
    MissingHeader(String),
}
