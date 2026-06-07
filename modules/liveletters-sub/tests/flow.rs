//! Тесты поведения `lltt sub` через публичный `run`.

mod common;

use liveletters_output::CommandContext;
use liveletters_sub::run;

fn tokens(args: &[&str]) -> liveletters_sub::Args {
    liveletters_sub::Args {
        tokens: args.iter().map(|s| s.to_string()).collect(),
    }
}

fn read_identity_subscriptions(home: &std::path::Path, name: &str) -> Vec<String> {
    let path = home.join("identities").join(format!("{name}.toml"));
    let text = std::fs::read_to_string(&path).expect("identity file");
    let cfg: liveletters_config::IdentityConfig = toml::from_str(&text).expect("identity parses");
    cfg.subscriptions()
        .iter()
        .map(|a| a.as_str().to_owned())
        .collect()
}

#[test]
fn subscribe_writes_local_subscriptions_and_outbox() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap();

    let subs = read_identity_subscriptions(home.path(), "bob");
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

    // output goes to stdout; we just check it doesn't panic and store is consistent
    run(&ctx, &tokens(&["list"])).unwrap();
}

#[test]
fn rm_removes_local_subscription_and_writes_unsubscribe() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap();
    run(&ctx, &tokens(&["rm", "alice-publish@example.org"])).unwrap();

    let subs = read_identity_subscriptions(home.path(), "bob");
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
    // Note: no add_identity, no init
    let ctx = CommandContext {
        home: home.path().to_path_buf(),
        state_home: home.path().to_path_buf(),
        identity_name: "default".to_owned(),
    };
    let err = run(&ctx, &tokens(&["alice-publish@example.org"])).unwrap_err();
    let _ = err;
}
