use liveletters_store::{
    CommentRecord, DeferredEventRecord, PostRecord, Store, UserSettingsRecord,
};

mod common;

fn save_post(store: &Store, id: &str, created_at: u64) {
    store
        .save_post_record(&PostRecord {
            post_id: id.into(),
            resource_id: "blog-1".into(),
            author_id: "alice".into(),
            created_at,
            body: "тело".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
}

fn save_comment(store: &Store, id: &str, post_id: &str) {
    store
        .save_comment_record(&CommentRecord {
            comment_id: id.into(),
            post_id: post_id.into(),
            parent_comment_id: None,
            author_id: "alice".into(),
            created_at: 1_710_000_100,
            body: "ответ".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
}

fn save_deferred(store: &Store, event_id: &str) {
    store
        .save_deferred_event_record(&DeferredEventRecord {
            event_id: event_id.into(),
            event_type: "post_created".into(),
            reason: "transient".into(),
            payload_json: "{}".into(),
        })
        .unwrap();
}

#[test]
fn count_posts_returns_zero_on_empty_db() {
    let (store, _tmp) = common::open_temp_store();
    assert_eq!(store.count_posts().unwrap(), 0);
    assert_eq!(store.newest_post_created_at().unwrap(), None);
}

#[test]
fn count_posts_reflects_inserted_rows() {
    let (store, _tmp) = common::open_temp_store();
    save_post(&store, "post-1", 1_710_000_000);
    save_post(&store, "post-2", 1_710_000_500);
    save_post(&store, "post-3", 1_710_001_000);
    assert_eq!(store.count_posts().unwrap(), 3);
}

#[test]
fn count_deferred_events_counts_only_deferred_table() {
    let (store, _tmp) = common::open_temp_store();
    save_deferred(&store, "ev-1");
    save_deferred(&store, "ev-2");
    save_post(&store, "post-1", 1_710_000_000);
    assert_eq!(store.count_deferred_events().unwrap(), 2);
    assert_eq!(store.count_posts().unwrap(), 1);
    assert_eq!(store.count_comments().unwrap(), 0);
    assert_eq!(store.count_outbox().unwrap(), 0);
}

#[test]
fn newest_post_created_at_returns_max() {
    let (store, _tmp) = common::open_temp_store();
    save_post(&store, "post-1", 1_710_000_000);
    save_post(&store, "post-2", 1_710_000_500);
    save_post(&store, "post-3", 1_710_001_000);
    assert_eq!(store.newest_post_created_at().unwrap(), Some(1_710_001_000));
}

#[test]
fn count_increments_after_save() {
    let (store, _tmp) = common::open_temp_store();
    assert_eq!(store.count_comments().unwrap(), 0);
    save_post(&store, "post-1", 1_710_000_000);
    save_comment(&store, "c-1", "post-1");
    save_comment(&store, "c-2", "post-1");
    assert_eq!(store.count_comments().unwrap(), 2);
}

#[test]
fn user_settings_roundtrip_via_save() {
    let (store, _tmp) = common::open_temp_store();
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "default".into(),
            nickname: "Алиса".into(),
            email_address: "alice@example.org".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: false,
        })
        .unwrap();
    let record = store.get_user_settings_record("default").unwrap().unwrap();
    assert_eq!(record.nickname, "Алиса");
    assert_eq!(record.language, "ru");
    assert!(!record.setup_completed);
}
