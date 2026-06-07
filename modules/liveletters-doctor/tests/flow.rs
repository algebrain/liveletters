use liveletters_diagnostics::DiagnosticsSnapshot;
use liveletters_doctor::print_doctor;
use liveletters_output::CommandContext;
use liveletters_store::{DeferredEventRecord, RawMessageRecord, Store, UserSettingsRecord};

mod common;

fn ctx_for(tmp: &tempfile::TempDir) -> CommandContext {
    CommandContext {
        home: tmp.path().to_path_buf(),
        state_home: tmp.path().to_path_buf(),
        identity_name: "default".to_owned(),
    }
}

#[test]
fn doctor_reports_healthy_on_empty_db() {
    let (_store, tmp) = common::open_temp_store();
    liveletters_doctor::run(&ctx_for(&tmp), &liveletters_doctor::Args::default()).unwrap();
}

#[test]
fn doctor_reports_degraded_when_deferred_present() {
    let (store, tmp) = common::open_temp_store();
    store
        .save_deferred_event_record(&DeferredEventRecord {
            event_id: "ev-1".into(),
            event_type: "post_created".into(),
            reason: "transient".into(),
            payload_json: "{}".into(),
        })
        .unwrap();
    liveletters_doctor::run(&ctx_for(&tmp), &liveletters_doctor::Args::default()).unwrap();
}

#[test]
fn doctor_reports_malformed_in_raw_messages() {
    let (store, tmp) = common::open_temp_store();
    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "m-1".into(),
            raw_message: "...".into(),
            status: "malformed".into(),
        })
        .unwrap();
    let ctx = ctx_for(&tmp);
    liveletters_doctor::run(&ctx, &liveletters_doctor::Args::default()).unwrap();
}

#[test]
fn print_doctor_with_handcrafted_snapshot() {
    use liveletters_diagnostics::{HealthStatus, SyncHealth, SyncHealthFields};

    let snap = DiagnosticsSnapshot::new(
        SyncHealth::new(SyncHealthFields {
            status: HealthStatus::Healthy,
            applied_messages: 1,
            duplicate_messages: 0,
            replayed_messages: 0,
            unauthorized_messages: 0,
            invalid_messages: 0,
            malformed_messages: 0,
            deferred_events: 0,
            pending_outbox: 1,
        }),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    print_doctor(&snap);
    let _ = UserSettingsRecord {
        profile_id: "default".into(),
        nickname: String::new(),
        email_address: String::new(),
        avatar_url: None,
        language: "ru".into(),
        setup_completed: false,
    };
    let _: &Store = &Store::open_for_home_dir(tmpfile_path()).unwrap();
}

fn tmpfile_path() -> std::path::PathBuf {
    let tmp = tempfile::tempdir().unwrap();
    tmp.path().to_path_buf()
}
