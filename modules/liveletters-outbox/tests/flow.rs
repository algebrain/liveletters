//! Тесты команды `lltt outbox` через публичный `run`.

mod common;

use liveletters_app_core::{
    AppCore, CreatePostFromIdentityCommand, GetPendingOutboxQuery, Identity, OutboxEntry,
    PendingOutbox, Visibility,
};
use liveletters_outbox::{Args, OutboxAction, print_summary, run};
use liveletters_store::{OutboxDelivery, UserSettingsRecord};

#[test]
fn outbox_list_empty_store_succeeds() {
    let home = common::TestHome::new();
    let ctx = home.ctx("alice");

    let args = Args {
        action: OutboxAction::List,
    };

    run(&ctx, &args).expect("outbox list on empty store should succeed");
}

#[test]
fn outbox_list_shows_pending_post_created() {
    let home = common::TestHome::new();
    let ctx = home.ctx("alice");

    let store = home.open_store();
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "alice".into(),
            nickname: "alice".into(),
            email_address: "alice@example.test".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: true,
        })
        .unwrap();
    let core = AppCore::new(&store);
    core.create_post_from_identity(CreatePostFromIdentityCommand {
        profile_id: "alice",
        identity: &Identity {
            publish: "alice-publish@example.org".to_owned(),
        },
        body: "Запись",
        visibility: Visibility::Public,
    })
    .unwrap();

    let args = Args {
        action: OutboxAction::List,
    };

    run(&ctx, &args).expect("outbox list should succeed");

    let pending = core
        .get_pending_outbox(GetPendingOutboxQuery)
        .expect("pending outbox");
    assert_eq!(pending.entries().len(), 1);
    assert_eq!(
        pending.entries()[0].resource_id,
        "alice-publish@example.org"
    );
}

#[test]
fn print_summary_works_with_empty_and_populated() {
    print_summary(&PendingOutbox::new(vec![]));

    let store = common::TestHome::new();
    let _ = store.path();

    let populated = PendingOutbox::new(vec![OutboxEntry {
        event_id: "post-created:post-1".to_owned(),
        event_type: "post_created".to_owned(),
        resource_id: "alice-publish@example.org".to_owned(),
        delivery: OutboxDelivery::ResourceSubscribers,
        message_body: "{}".to_owned(),
    }]);
    print_summary(&populated);
}
