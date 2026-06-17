//! Тесты `liveletters-doctor::print_doctor_verbose` (этап 8.27).

use liveletters_diagnostics::DiagnosticsReader;
use liveletters_doctor::print_doctor_verbose;
use liveletters_store::DeferredEventRecord;

mod common;

#[test]
fn verbose_empty_db_prints_all_three_sections() {
    let (store, tmp) = common::open_temp_store();
    let reader = DiagnosticsReader::new(&store);
    let snap = reader.build_snapshot().unwrap();

    print_doctor_verbose(&snap, &store, tmp.path()).expect("verbose print");
}

#[test]
fn verbose_shows_deferred_events() {
    let (store, tmp) = common::open_temp_store();
    store
        .save_deferred_event_record(&DeferredEventRecord {
            event_id: "ev-1".to_owned(),
            event_type: "post_created".to_owned(),
            reason: "transient-network".to_owned(),
            payload_json: "{}".to_owned(),
            origin: "Alice <alice@example.test>".to_owned(),
        })
        .unwrap();

    let reader = DiagnosticsReader::new(&store);
    let snap = reader.build_snapshot().unwrap();
    print_doctor_verbose(&snap, &store, tmp.path()).expect("verbose print");
}

#[test]
fn verbose_lists_identities_and_current_user() {
    let (store, tmp) = common::open_temp_store();
    let home = tmp.path();

    let identities_dir = home.join("identities");
    std::fs::create_dir_all(&identities_dir).unwrap();
    std::fs::write(identities_dir.join("alice.toml"), "display_name = \"А\"\n").unwrap();
    std::fs::write(identities_dir.join("bob.toml"), "display_name = \"Б\"\n").unwrap();
    std::fs::write(home.join("current-user"), "alice\n").unwrap();

    let reader = DiagnosticsReader::new(&store);
    let snap = reader.build_snapshot().unwrap();
    print_doctor_verbose(&snap, &store, home).expect("verbose print");
}
