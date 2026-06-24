use std::fs;
use std::path::{Path, PathBuf};

use liveletters_mime::{ReceivedEmail, parse_email};
use liveletters_store::Store;
use liveletters_sync::{SyncEngine, SyncMessageOutcome};

use crate::InboxError;

pub fn run(home: &Path, files: &[PathBuf]) -> Result<(), InboxError> {
    let store = Store::open_for_home_dir(home)?;

    // Профиль_id берём из `current-user` файла (если он есть).
    // Без профиля id `decline_pending` / `accept_pending` используют «default»,
    // что работает не для всех пользователей. Здесь оставляем как есть —
    // inbox import работает на текущий профиль (тот, кто выполняет команду).
    let profile_id = read_current_profile_id(home);
    let engine = SyncEngine::new(&store).with_profile_id(&profile_id);

    let mut total_applied = 0;
    let mut total_duplicate = 0;
    let mut total_deferred = 0;
    let mut total_filtered = 0;
    let mut total_rejected = 0;

    for file in files {
        if !file.exists() {
            return Err(InboxError::FileNotFound(file.clone()));
        }
        let raw = fs::read_to_string(file)?;
        let parsed = parse_email(&raw)?;
        let message_id = parsed
            .header("Message-ID")
            .or_else(|| parsed.header("Message-Id"))
            .unwrap_or("")
            .to_owned();
        let received = ReceivedEmail {
            message_id,
            raw_message: raw,
        };
        let report = engine.ingest_batch(vec![received])?;
        for outcome in report.outcomes() {
            match outcome {
                SyncMessageOutcome::Applied { event_id, .. } => {
                    println!("{}: применено ({event_id})", file.display());
                    total_applied += 1;
                }
                SyncMessageOutcome::Duplicate { event_id, .. } => {
                    println!("{}: дубликат ({event_id})", file.display());
                    total_duplicate += 1;
                }
                SyncMessageOutcome::Deferred { reason, .. } => {
                    println!("{}: отложено ({reason})", file.display());
                    total_deferred += 1;
                }
                SyncMessageOutcome::Filtered { reason, .. } => {
                    println!("{}: отфильтровано ({reason})", file.display());
                    total_filtered += 1;
                }
                SyncMessageOutcome::Malformed { reason, .. } => {
                    println!("{}: отклонено ({reason})", file.display());
                    total_rejected += 1;
                }
                SyncMessageOutcome::Replay { reason, .. }
                | SyncMessageOutcome::Unauthorized { reason, .. }
                | SyncMessageOutcome::Invalid { reason, .. }
                | SyncMessageOutcome::RateLimited { reason, .. } => {
                    println!("{}: отклонено ({reason})", file.display());
                    total_rejected += 1;
                }
            }
        }
    }

    println!();
    println!("применено: {total_applied}");
    println!("дубликатов: {total_duplicate}");
    println!("отложено:   {total_deferred}");
    println!("отфильтровано: {total_filtered}");
    println!("отклонено:  {total_rejected}");

    Ok(())
}

fn read_current_profile_id(state_home: &Path) -> String {
    // `current-user` лежит в корне LIVELETTERS_HOME, не в `users/<name>/`.
    // `state_home` = `<home>/users/<name>`, поэтому поднимаемся на два уровня.
    let home_root = state_home
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(state_home);
    let path = home_root.join("current-user");
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string())
}
