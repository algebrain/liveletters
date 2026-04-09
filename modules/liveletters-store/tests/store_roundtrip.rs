use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use liveletters_store::{
    CommentRecord, DeferredEventRecord, OutboxRecord, PostRecord, RawEventRecord, RawMessageRecord,
    Store, StorePaths,
};

fn temp_home_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("liveletters-home-{unique}"));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn in_memory_store_starts_with_empty_state() {
    let store = Store::open_in_memory().unwrap();

    assert!(store.list_posts().unwrap().is_empty());
    assert!(store.list_comments_for_post("post-1").unwrap().is_empty());
    assert!(store.list_outbox_records().unwrap().is_empty());
    assert!(store.list_raw_message_records().unwrap().is_empty());
    assert!(store.list_raw_event_records().unwrap().is_empty());
    assert!(store.list_deferred_event_records().unwrap().is_empty());
}

#[test]
fn saved_post_can_be_read_back() {
    let store = Store::open_in_memory().unwrap();

    store
        .save_post_record(&PostRecord {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            author_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Первая запись".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();

    let posts = store.list_posts().unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].post_id, "post-1");
    assert_eq!(posts[0].body, "Первая запись");
    assert_eq!(posts[0].visibility, "public");
}

#[test]
fn saved_comment_is_returned_for_its_post() {
    let store = Store::open_in_memory().unwrap();

    store
        .save_post_record(&PostRecord {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            author_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Первая запись".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();

    store
        .save_comment_record(&CommentRecord {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            parent_comment_id: Some("comment-root".into()),
            author_id: "alice".into(),
            created_at: 1_710_000_100,
            body: "Ответ".into(),
            visibility: "friends_only".into(),
            hidden: false,
        })
        .unwrap();

    let comments = store.list_comments_for_post("post-1").unwrap();

    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].comment_id, "comment-1");
    assert_eq!(comments[0].parent_comment_id.as_deref(), Some("comment-root"));
    assert_eq!(comments[0].body, "Ответ");
}

#[test]
fn store_paths_use_home_scoped_liveletters_directory() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    assert_eq!(paths.data_dir(), home_dir.join(".liveletters"));
    assert_eq!(
        paths.database_path(),
        home_dir.join(".liveletters").join("liveletters.sqlite3")
    );

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn file_store_persists_records_under_temp_home() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    {
        let store = Store::open_at(paths.database_path()).unwrap();
        store
            .save_post_record(&PostRecord {
                post_id: "post-1".into(),
                resource_id: "blog-1".into(),
                author_id: "alice".into(),
                created_at: 1_710_000_000,
                body: "Первая запись".into(),
                visibility: "public".into(),
                hidden: false,
            })
            .unwrap();
    }

    assert!(paths.database_path().exists());

    let reopened = Store::open_at(paths.database_path()).unwrap();
    let posts = reopened.list_posts().unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].post_id, "post-1");

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn outbox_records_can_be_saved_and_listed() {
    let store = Store::open_in_memory().unwrap();

    store
        .save_outbox_record(&OutboxRecord {
            event_id: "event-1".into(),
            event_type: "post_created".into(),
            resource_id: "blog-1".into(),
            message_body: "{\"kind\":\"post_created\"}".into(),
        })
        .unwrap();

    let outbox = store.list_outbox_records().unwrap();

    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_id, "event-1");
    assert_eq!(outbox[0].event_type, "post_created");
}

#[test]
fn raw_message_and_event_journals_can_be_saved() {
    let store = Store::open_in_memory().unwrap();

    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "message-1".into(),
            raw_message: "raw email".into(),
            status: "applied".into(),
        })
        .unwrap();
    store
        .save_raw_event_record(&RawEventRecord {
            event_id: "event-1".into(),
            event_type: "post_created".into(),
            resource_id: "blog-1".into(),
            payload_json: "{\"kind\":\"post_created\"}".into(),
            apply_status: "applied".into(),
            failure_reason: None,
        })
        .unwrap();

    assert_eq!(store.list_raw_message_records().unwrap().len(), 1);
    assert_eq!(store.list_raw_event_records().unwrap().len(), 1);
    assert!(store.has_raw_event("event-1").unwrap());
    assert_eq!(store.list_raw_event_records().unwrap()[0].apply_status, "applied");
}

#[test]
fn deferred_events_can_be_saved_and_listed() {
    let store = Store::open_in_memory().unwrap();

    store
        .save_deferred_event_record(&DeferredEventRecord {
            event_id: "event-2".into(),
            event_type: "comment_created".into(),
            reason: "missing_post".into(),
            payload_json: "{\"kind\":\"comment_created\"}".into(),
        })
        .unwrap();

    let deferred = store.list_deferred_event_records().unwrap();

    assert_eq!(deferred.len(), 1);
    assert_eq!(deferred[0].reason, "missing_post");
}

#[test]
fn deferred_event_can_be_deleted_after_reprocessing() {
    let store = Store::open_in_memory().unwrap();

    store
        .save_deferred_event_record(&DeferredEventRecord {
            event_id: "event-2".into(),
            event_type: "comment_created".into(),
            reason: "missing_post".into(),
            payload_json: "{\"kind\":\"comment_created\"}".into(),
        })
        .unwrap();

    store.delete_deferred_event_record("event-2").unwrap();

    assert!(store.list_deferred_event_records().unwrap().is_empty());
}
