use liveletters_inbox::{Args, InboxAction, ListArgs};
use liveletters_output::CommandContext;
use liveletters_store::RawMessageRecord;

mod common;

fn ctx_for(tmp: &tempfile::TempDir) -> CommandContext {
    CommandContext {
        home: tmp.path().to_path_buf(),
        state_home: tmp.path().to_path_buf(),
        identity_name: "default".to_owned(),
    }
}

fn save_message(store: &liveletters_store::Store, id: &str, status: &str) {
    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: id.into(),
            raw_message: "From: alice@example.test\n\nhello".into(),
            status: status.into(),
        })
        .unwrap();
}

#[test]
fn list_on_empty_db_prints_zero() {
    let (_store, tmp) = common::open_temp_store();
    let args = Args {
        action: InboxAction::List(ListArgs {
            status: None,
            limit: 20,
        }),
    };
    liveletters_inbox::run(&ctx_for(&tmp), &args).unwrap();
}

#[test]
fn list_filters_by_status() {
    let (store, tmp) = common::open_temp_store();
    save_message(&store, "m-1", "applied");
    save_message(&store, "m-2", "duplicate");
    save_message(&store, "m-3", "applied");
    save_message(&store, "m-4", "malformed");
    let args = Args {
        action: InboxAction::List(ListArgs {
            status: Some("applied".into()),
            limit: 20,
        }),
    };
    liveletters_inbox::run(&ctx_for(&tmp), &args).unwrap();
}

#[test]
fn list_rejects_unknown_status() {
    let (_store, tmp) = common::open_temp_store();
    let args = Args {
        action: InboxAction::List(ListArgs {
            status: Some("nonsense".into()),
            limit: 20,
        }),
    };
    let err = liveletters_inbox::run(&ctx_for(&tmp), &args).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("nonsense"), "msg: {msg}");
    assert!(msg.contains("допустимые"), "msg: {msg}");
}

#[test]
fn list_respects_limit() {
    let (store, tmp) = common::open_temp_store();
    for i in 0..5 {
        save_message(&store, &format!("m-{i}"), "applied");
    }
    let args = Args {
        action: InboxAction::List(ListArgs {
            status: None,
            limit: 2,
        }),
    };
    liveletters_inbox::run(&ctx_for(&tmp), &args).unwrap();
}
