//! Печать сводки состояния синхронизации.

use std::path::Path;

use liveletters_diagnostics::{DiagnosticsSnapshot, HealthStatus};
use liveletters_output::print_kv;
use liveletters_store::Store;

use crate::error::DoctorError;

pub fn print_doctor(snap: &DiagnosticsSnapshot) {
    let h = snap.sync_health();
    let status_ru = match h.status() {
        HealthStatus::Healthy => "здоров",
        HealthStatus::Degraded => "деградирован",
    };
    print_kv(&[
        ("здоровье", status_ru),
        ("Applied", &h.applied_messages().to_string()),
        ("Duplicate", &h.duplicate_messages().to_string()),
        ("Replay", &h.replayed_messages().to_string()),
        ("Unauthorized", &h.unauthorized_messages().to_string()),
        ("Invalid", &h.invalid_messages().to_string()),
        ("Malformed", &h.malformed_messages().to_string()),
        ("Deferred", &h.deferred_events().to_string()),
        ("Outbox (исходящих)", &h.pending_outbox().to_string()),
    ]);
}

const VERBOSE_DEFERRED_LIMIT: usize = 10;
const VERBOSE_TABLES: &[&str] = &[
    "posts",
    "comments",
    "outbox",
    "raw_messages",
    "deferred_events",
    "subscriptions",
];

pub fn print_doctor_verbose(
    snap: &DiagnosticsSnapshot,
    store: &Store,
    home: &Path,
) -> Result<(), DoctorError> {
    print_doctor(snap);

    println!();
    println!("--- deferred_events (последние {VERBOSE_DEFERRED_LIMIT}) ---");
    let deferred = store.list_deferred_events(VERBOSE_DEFERRED_LIMIT)?;
    if deferred.is_empty() {
        println!("(нет)");
    } else {
        for d in &deferred {
            println!("  - {}: {}", d.event_id, d.reason);
        }
    }

    println!();
    println!("--- identities ---");
    let identities_dir = home.join("identities");
    let current = read_current_user(home).unwrap_or_else(|_| "(не задан)".to_owned());
    match std::fs::read_dir(&identities_dir) {
        Ok(rd) => {
            let mut names: Vec<String> = rd
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .filter(|n| n.ends_with(".toml"))
                .map(|n| n.trim_end_matches(".toml").to_owned())
                .collect();
            names.sort();
            println!(
                "  {} конфигов: {}; текущий: {}",
                names.len(),
                names.join(", "),
                current
            );
        }
        Err(_) => println!("  (каталог identities/ отсутствует)"),
    }

    println!();
    println!("--- таблицы ---");
    for table in VERBOSE_TABLES {
        let bytes = store.table_size(table).unwrap_or(0);
        println!("  {table}: {bytes} B");
    }
    Ok(())
}

fn read_current_user(home: &Path) -> Result<String, std::io::Error> {
    std::fs::read_to_string(home.join("current-user")).map(|s| s.trim().to_owned())
}
