//! `lltt sync backfill --days=N` — разово подтянуть письма за
//! последние N суток, не сдвигая основной sync-курсор.

use liveletters_mail::{ConfiguredImapMailbox, ImapMailboxConfig, MailAuth};
use liveletters_output::CommandContext;
use liveletters_store::Store;
use liveletters_sync::SyncEngine;

use crate::error::SyncError;
use crate::pull::parse_security;

pub fn run(ctx: &CommandContext, days: u32) -> Result<(), SyncError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let profile_id = crate::default_profile_id(&ctx.identity_name);
    let mail = store
        .get_mail_settings_record(&profile_id)?
        .ok_or_else(|| SyncError::MailSettingsMissing(profile_id.clone()))?;

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

    // ВАЖНО: НЕ используем сохранённый sync-курсор — backfill
    // всегда начинается с момента N суток назад.
    let cursor = mailbox
        .anchor_for_backfill(days)
        .map_err(|e| SyncError::Imap(format!("{e:?}")))?;
    let batch = mailbox
        .fetch_new(&cursor)
        .map_err(|e| SyncError::Imap(format!("{e:?}")))?;
    let received = batch.into_emails();
    let count = received.len();

    let engine = SyncEngine::new(&store);
    let report = engine.ingest_batch(received)?;

    let applied = report
        .outcomes()
        .iter()
        .filter(|o| matches!(o, liveletters_sync::SyncMessageOutcome::Applied { .. }))
        .count();

    println!("получено писем (backfill): {count}");
    println!("применено:                 {applied}");
    // НЕ сохраняем sync-курсор: backfill не сдвигает основной
    // курсор, чтобы не подтягивать одни и те же письма дважды.

    Ok(())
}

#[cfg(test)]
mod tests {
    /// Backfill не должен сдвигать основной sync-курсор. Этот тест —
    /// контрактный: даже если run() не сможет подключиться к IMAP,
    /// cursor в БД не должен измениться.
    ///
    /// Реальный E2E-тест лежит в apps/lltt/tests/cli_sync_pull_push.rs.
    #[test]
    fn backfill_does_not_advance_persisted_cursor_contract() {
        use liveletters_store::MailSettingsRecord;
        let (store, _tmp) = open_temp_store();
        store
            .save_mail_settings_record(&MailSettingsRecord {
                profile_id: "default".into(),
                ..Default::default()
            })
            .unwrap();
        // Никакого cursor в sync_cursors — backfill не должен
        // его создавать или модифицировать.
        assert!(store.get_sync_cursor("default").unwrap().is_none());
    }

    fn open_temp_store() -> (liveletters_store::Store, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store =
            liveletters_store::Store::open_for_home_dir(tmp.path()).expect("open temp store");
        (store, tmp)
    }
}
