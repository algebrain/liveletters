use liveletters_app_core::{
    AppCore, CommentSummary, CreateCommentCommand, CreatePostCommand, GetHomeFeedQuery,
    GetPendingOutboxQuery, GetPostThreadQuery, PostSummary, ReprocessDeferredEventsCommand,
};
use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{DomainEventPayload, decode_message};
use liveletters_protocol::{MessageEnvelope, ProtocolMessage};
use liveletters_store::Store;
use liveletters_sync::SyncEngine;

#[test]
fn creates_post_and_exposes_it_in_home_feed() {
    let store = Store::open_in_memory().expect("store should open");
    let app = AppCore::new(&store);

    let created = app
        .create_post(CreatePostCommand {
            post_id: "post-1",
            resource_id: "blog-1",
            author_id: "alice",
            created_at: 1,
            body: "First post",
        })
        .expect("post should be created");

    assert_eq!(created.post().id().as_str(), "post-1");

    let feed = app
        .get_home_feed(GetHomeFeedQuery)
        .expect("feed should load");

    assert_eq!(
        feed.posts(),
        &[PostSummary {
            post_id: "post-1".to_owned(),
            resource_id: "blog-1".to_owned(),
            author_id: "alice".to_owned(),
            created_at: 1,
            body: "First post".to_owned(),
            visibility: "public".to_owned(),
            hidden: false,
        }]
    );

    let outbox = app
        .get_pending_outbox(GetPendingOutboxQuery)
        .expect("outbox should load");

    assert_eq!(outbox.entries().len(), 1);
    assert_eq!(outbox.entries()[0].event_type, "post_created");

    let decoded = decode_message(&outbox.entries()[0].message_body).expect("message should decode");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[test]
fn creates_comment_and_exposes_thread_for_post() {
    let store = Store::open_in_memory().expect("store should open");
    let app = AppCore::new(&store);

    app.create_post(CreatePostCommand {
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "alice",
        created_at: 1,
        body: "First post",
    })
    .expect("post should be created");

    let created = app
        .create_comment(CreateCommentCommand {
            comment_id: "comment-1",
            post_id: "post-1",
            parent_comment_id: None,
            author_id: "bob",
            created_at: 2,
            body: "First comment",
        })
        .expect("comment should be created");

    assert_eq!(created.comment().id().as_str(), "comment-1");

    let thread = app
        .get_post_thread(GetPostThreadQuery { post_id: "post-1" })
        .expect("thread should load");

    assert_eq!(
        thread.post(),
        &PostSummary {
            post_id: "post-1".to_owned(),
            resource_id: "blog-1".to_owned(),
            author_id: "alice".to_owned(),
            created_at: 1,
            body: "First post".to_owned(),
            visibility: "public".to_owned(),
            hidden: false,
        }
    );
    assert_eq!(
        thread.comments(),
        &[CommentSummary {
            comment_id: "comment-1".to_owned(),
            post_id: "post-1".to_owned(),
            parent_comment_id: None,
            author_id: "bob".to_owned(),
            created_at: 2,
            body: "First comment".to_owned(),
            visibility: "public".to_owned(),
            hidden: false,
        }]
    );

    let outbox = app
        .get_pending_outbox(GetPendingOutboxQuery)
        .expect("outbox should load");

    assert_eq!(outbox.entries().len(), 2);
    assert_eq!(outbox.entries()[1].event_type, "comment_created");
}

#[test]
fn rejects_comment_for_missing_post() {
    let store = Store::open_in_memory().expect("store should open");
    let app = AppCore::new(&store);

    let error = app
        .create_comment(CreateCommentCommand {
            comment_id: "comment-1",
            post_id: "missing-post",
            parent_comment_id: None,
            author_id: "bob",
            created_at: 2,
            body: "First comment",
        })
        .expect_err("missing post should be rejected");

    assert!(matches!(
        error,
        liveletters_app_core::AppCoreError::PostNotFound { post_id }
        if post_id == "missing-post"
    ));
}

#[test]
fn hides_post_and_keeps_hidden_state_in_feed() {
    let store = Store::open_in_memory().expect("store should open");
    let app = AppCore::new(&store);

    app.create_post(CreatePostCommand {
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "alice",
        created_at: 1,
        body: "First post",
    })
    .expect("post should be created");

    let hidden = app
        .hide_post(liveletters_app_core::HidePostCommand {
            post_id: "post-1",
            actor_id: "alice",
            created_at: 2,
        })
        .expect("post should be hidden");

    assert!(hidden.post().is_hidden());

    let feed = app
        .get_home_feed(GetHomeFeedQuery)
        .expect("feed should load");

    assert!(feed.posts()[0].hidden);

    let outbox = app
        .get_pending_outbox(GetPendingOutboxQuery)
        .expect("outbox should load");
    assert_eq!(outbox.entries()[1].event_type, "post_hidden");
}

#[test]
fn edits_comment_and_returns_updated_thread() {
    let store = Store::open_in_memory().expect("store should open");
    let app = AppCore::new(&store);

    app.create_post(CreatePostCommand {
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "alice",
        created_at: 1,
        body: "First post",
    })
    .expect("post should be created");

    app.create_comment(CreateCommentCommand {
        comment_id: "comment-1",
        post_id: "post-1",
        parent_comment_id: None,
        author_id: "bob",
        created_at: 2,
        body: "First comment",
    })
    .expect("comment should be created");

    let edited = app
        .edit_comment(liveletters_app_core::EditCommentCommand {
            comment_id: "comment-1",
            actor_id: "bob",
            created_at: 3,
            body: "Edited comment",
        })
        .expect("comment should be edited");

    assert_eq!(edited.comment().body().as_str(), "Edited comment");

    let thread = app
        .get_post_thread(GetPostThreadQuery { post_id: "post-1" })
        .expect("thread should load");

    assert_eq!(thread.comments()[0].body, "Edited comment");

    let outbox = app
        .get_pending_outbox(GetPendingOutboxQuery)
        .expect("outbox should load");
    assert_eq!(outbox.entries()[2].event_type, "comment_edited");

    let decoded = decode_message(&outbox.entries()[2].message_body).expect("message should decode");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::CommentEdited { body, .. } if body == "Edited comment"
    ));
}

#[test]
fn reprocesses_deferred_events_through_app_core_orchestration() {
    let store = Store::open_in_memory().expect("store should open");
    let sync = SyncEngine::new(&store);

    let deferred_message = ProtocolMessage::new(
        MessageEnvelope::new("1", "comment_created", "blog-1", "event-comment-1").unwrap(),
        "Комментарий раньше поста",
        DomainEventPayload::CommentCreated {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            parent_comment_id: None,
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 2,
            visibility: "public".into(),
        },
    )
    .unwrap();
    let deferred_email = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Deferred comment",
        &deferred_message,
    )
    .unwrap();

    sync.ingest_batch(vec![ReceivedEmail {
        message_id: "message-comment-1".into(),
        raw_message: deferred_email.raw_message,
    }])
    .expect("deferred message should ingest");

    let app = AppCore::new(&store);
    app.create_post(CreatePostCommand {
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "alice",
        created_at: 1,
        body: "First post",
    })
    .expect("post should be created");

    let result = app
        .reprocess_deferred_events(ReprocessDeferredEventsCommand)
        .expect("reprocessing should succeed");

    assert_eq!(result.summary().applied, 1);
    assert_eq!(result.summary().still_deferred, 0);

    let thread = app
        .get_post_thread(GetPostThreadQuery { post_id: "post-1" })
        .expect("thread should load");
    assert_eq!(thread.comments().len(), 1);
    assert_eq!(thread.comments()[0].comment_id, "comment-1");
}
