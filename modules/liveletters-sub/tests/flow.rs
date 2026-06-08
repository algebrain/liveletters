//! Тесты поведения `lltt sub` через публичный `run`.

mod common;

use liveletters_output::CommandContext;
use liveletters_sub::run;

fn tokens(args: &[&str]) -> liveletters_sub::Args {
    liveletters_sub::Args {
        tokens: args.iter().map(|s| s.to_string()).collect(),
    }
}
fn read_local_subscriptions(home: &std::path::Path, name: &str) -> Vec<String> {
    let store = liveletters_store::Store::open_for_home_dir(home).unwrap();
    store.list_local_subscriptions(name).unwrap()
}

#[test]
fn subscribe_writes_local_subscriptions_and_outbox() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap();

    let subs = read_local_subscriptions(home.path(), "bob");
    assert_eq!(subs, vec!["alice-publish@example.org"]);

    let store = home.open_store();
    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    assert!(
        outbox[0]
            .event_id
            .starts_with("subscription:alice-publish@example.org:"),
        "event_id={}",
        outbox[0].event_id
    );
}

#[test]
fn subscribe_rejects_invalid_address() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    let err = run(&ctx, &tokens(&["not-an-address"])).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("@") || msg.contains("адрес") || msg.contains("адреса"),
        "got: {msg}"
    );
}

#[test]
fn list_shows_subscribed_and_owned() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap();

    run(&ctx, &tokens(&["list"])).unwrap();
}

#[test]
fn rm_removes_local_subscription_and_writes_unsubscribe() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap();
    run(&ctx, &tokens(&["rm", "alice-publish@example.org"])).unwrap();

    let subs = read_local_subscriptions(home.path(), "bob");
    assert!(subs.is_empty());

    let store = home.open_store();
    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 2);
    assert!(
        outbox
            .iter()
            .all(|r| r.event_id.starts_with("subscription:")
                || r.event_id.starts_with("unsubscription:"))
    );
}

#[test]
fn empty_args_errors() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    let err = run(&ctx, &tokens(&[])).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("адрес") || msg.contains("require") || msg.contains("требует"),
        "got: {msg}"
    );
}

#[test]
fn no_init_errors_when_store_missing() {
    let home = common::TestHome::new();
    let ctx = CommandContext {
        home: home.path().to_path_buf(),
        state_home: home.path().to_path_buf(),
        identity_name: "default".to_owned(),
    };
    let err = run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap_err();
    let _ = err;
}
