//! `lltt sync pull` — забрать новые письма с IMAP, прогнать через
//! `liveletters_sync::SyncEngine::ingest_batch`, обновить курсор.

use liveletters_mail::{
    ConfiguredImapMailbox, ImapMailboxConfig, MailAuth, MailSecurity, MailboxCursor,
};
use liveletters_output::CommandContext;
use liveletters_store::Store;
use liveletters_sync::{SyncEngine, SyncMessageOutcome, SyncReport};

use crate::error::SyncError;

pub fn run(ctx: &CommandContext) -> Result<(), SyncError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let profile_id = default_profile_id(&ctx.identity_name);
    let mail = store
        .get_mail_settings_record(&profile_id)?
        .ok_or_else(|| SyncError::MailSettingsMissing(profile_id.clone()))?;

    let cursor_uid = store.get_sync_cursor(&profile_id)?.unwrap_or(0);
    let mailbox = ConfiguredImapMailbox::new(ImapMailboxConfig::new(
        &mail.imap_host,
        mail.imap_port,
        &mail.imap_mailbox,
        parse_security(&mail.imap_security)?,
        MailAuth::Password {
            username: mail.imap_username.clone(),
            password: mail.imap_password,
        },
    ));

    // Первый запуск (cursor_uid == 0): используем SINCE, чтобы
    // пропустить старые письма за пределами initial_lookback_days.
    let cursor = if cursor_uid == 0 {
        let since_uid = mailbox
            .find_min_uid_since_days(mail.initial_lookback_days)
            .map_err(|e| SyncError::Imap(format!("{e:?}")))?;
        MailboxCursor::start_with_since_uid(since_uid.max(1))
    } else {
        MailboxCursor::from_last_seen_uid(cursor_uid)
    };

    let batch = mailbox
        .fetch_new(&cursor)
        .map_err(|e| SyncError::Imap(format!("{e:?}")))?;
    let received = batch.into_emails();

    println!("получено писем:       {}", received.len());
    liveletters_log::log_info(format!(
        "sync.pull profile={profile_id} received={}",
        received.len()
    ));

    let next_uid = compute_next_cursor_uid(cursor_uid, &received);

    let engine = SyncEngine::new(&store);
    let report = engine.ingest_batch(received)?;
    let counts = tally(&report);

    println!("применено событий:    {}", counts.applied);
    println!("дубликатов:           {}", counts.duplicates);
    println!("некорректных писем:   {}", counts.malformed);
    liveletters_log::log_info(format!(
        "sync.pull summary profile={profile_id} applied={} duplicates={} malformed={}",
        counts.applied, counts.duplicates, counts.malformed,
    ));

    store.save_sync_cursor(&profile_id, next_uid)?;

    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct OutcomeCounts {
    pub applied: usize,
    pub duplicates: usize,
    pub malformed: usize,
}

pub fn tally(report: &SyncReport) -> OutcomeCounts {
    let mut counts = OutcomeCounts::default();
    for outcome in report.outcomes() {
        match outcome {
            SyncMessageOutcome::Applied { .. } => counts.applied += 1,
            SyncMessageOutcome::Duplicate { .. } => counts.duplicates += 1,
            SyncMessageOutcome::Malformed { .. } => counts.malformed += 1,
            _ => {}
        }
    }
    counts
}

pub fn compute_next_cursor_uid(prev: u64, received: &[liveletters_mail::ReceivedEmail]) -> u64 {
    let max_in_batch = received
        .iter()
        .filter_map(|email| {
            email
                .message_id
                .strip_prefix("imap-uid-")
                .and_then(|s| s.parse::<u64>().ok())
        })
        .max();
    max_in_batch.map_or(prev, |uid| uid.max(prev))
}

pub fn parse_security(s: &str) -> Result<MailSecurity, SyncError> {
    let normalized = s
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', '/', ' '], "");
    match normalized.as_str() {
        "none" => Ok(MailSecurity::None),
        "starttls" => Ok(MailSecurity::StartTls),
        "tls" | "ssl" | "ssltls" => Ok(MailSecurity::Tls),
        other => Err(SyncError::UnknownMailSecurity(other.to_owned())),
    }
}

fn default_profile_id(name: &str) -> String {
    if name.is_empty() {
        "default".to_owned()
    } else {
        name.to_owned()
    }
}
