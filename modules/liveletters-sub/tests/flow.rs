//! Тесты поведения `lltt sub` через публичный `run`.

mod common;

use liveletters_output::CommandContext;
use liveletters_sub::run;

fn tokens(args: &[&str]) -> liveletters_sub::Args {
    liveletters_sub::Args {
        tokens: args.iter().map(|s| s.to_string()).collect(),
    }
}
fn read_pending_subscriptions(home: &std::path::Path, name: &str) -> Vec<String> {
    let store = liveletters_store::Store::open_for_home_dir(home).unwrap();
    store
        .list_pending_subscriptions(name)
        .unwrap()
        .into_iter()
        .map(|r| r.resource_address)
        .collect()
}

#[test]
fn subscribe_writes_pending_and_outbox() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap();

    // pending_subscriptions содержит запись
    let pending = read_pending_subscriptions(home.path(), "bob");
    assert_eq!(pending, vec!["alice-publish@example.org".to_string()]);

    // local_subscriptions пуст — будет заполнен при SubscriptionConfirmed
    let store = home.open_store();
    let local = store.list_local_subscriptions("bob").unwrap();
    assert!(local.is_empty());

    // outbox получил subscription_requested
    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_type, "subscription_requested");
    assert!(
        outbox[0]
            .event_id
            .starts_with("subscription:alice-publish@example.org:"),
        "event_id={}",
        outbox[0].event_id
    );
    assert!(
        outbox[0].message_id.is_some(),
        "Message-ID должен быть заполнен"
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
fn rm_removes_pending_subscription_and_writes_unsubscribe() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap();
    run(&ctx, &tokens(&["rm", "alice-publish@example.org"])).unwrap();

    // pending_subscriptions пуст — отписка отменяет ожидающую подписку
    let pending = read_pending_subscriptions(home.path(), "bob");
    assert!(pending.is_empty());

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
