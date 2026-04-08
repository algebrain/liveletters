use liveletters_domain::{
    AccountId, Comment, CommentBody, CommentCreated, CommentId, DomainError, EventId, Post,
    PostBody, PostCreated, PostId, ResourceId, Timestamp, Visibility,
};

#[test]
fn post_id_rejects_blank_value() {
    let error = PostId::new("   ").expect_err("blank post id must be rejected");

    assert_eq!(error, DomainError::BlankIdentifier("post_id"));
}

#[test]
fn comment_id_rejects_blank_value() {
    let error = CommentId::new("").expect_err("blank comment id must be rejected");

    assert_eq!(error, DomainError::BlankIdentifier("comment_id"));
}

#[test]
fn post_creation_keeps_ids_body_and_visibility() {
    let post = Post::new(
        PostId::new("post-1").unwrap(),
        ResourceId::new("blog-1").unwrap(),
        AccountId::new("alice").unwrap(),
        Timestamp::from_unix_seconds(1_710_000_000),
        PostBody::new("Первая запись").unwrap(),
        Visibility::FriendsOnly,
    )
    .unwrap();

    assert_eq!(post.id().as_str(), "post-1");
    assert_eq!(post.resource_id().as_str(), "blog-1");
    assert_eq!(post.author_id().as_str(), "alice");
    assert_eq!(post.created_at().as_unix_seconds(), 1_710_000_000);
    assert_eq!(post.body().as_str(), "Первая запись");
    assert_eq!(post.visibility(), Visibility::FriendsOnly);
    assert!(!post.is_hidden());
}

#[test]
fn post_body_rejects_blank_value() {
    let error = PostBody::new("   ").expect_err("blank post body must be rejected");

    assert_eq!(error, DomainError::BlankBody("post_body"));
}

#[test]
fn comment_creation_keeps_parent_link() {
    let comment = Comment::new(
        CommentId::new("comment-1").unwrap(),
        PostId::new("post-1").unwrap(),
        Some(CommentId::new("comment-root").unwrap()),
        AccountId::new("alice").unwrap(),
        Timestamp::from_unix_seconds(1_710_000_100),
        CommentBody::new("Ответ").unwrap(),
        Visibility::Public,
    )
    .unwrap();

    assert_eq!(comment.id().as_str(), "comment-1");
    assert_eq!(comment.post_id().as_str(), "post-1");
    assert_eq!(
        comment.parent_comment_id().map(CommentId::as_str),
        Some("comment-root")
    );
    assert_eq!(comment.author_id().as_str(), "alice");
    assert_eq!(comment.created_at().as_unix_seconds(), 1_710_000_100);
    assert_eq!(comment.body().as_str(), "Ответ");
}

#[test]
fn comment_rejects_blank_body() {
    let error = CommentBody::new("  ").expect_err("blank comment body must be rejected");

    assert_eq!(error, DomainError::BlankBody("comment_body"));
}

#[test]
fn post_can_be_hidden() {
    let post = Post::new(
        PostId::new("post-1").unwrap(),
        ResourceId::new("blog-1").unwrap(),
        AccountId::new("alice").unwrap(),
        Timestamp::from_unix_seconds(1_710_000_000),
        PostBody::new("Первая запись").unwrap(),
        Visibility::Public,
    )
    .unwrap()
    .hide();

    assert!(post.is_hidden());
}

#[test]
fn comment_can_be_hidden() {
    let comment = Comment::new(
        CommentId::new("comment-1").unwrap(),
        PostId::new("post-1").unwrap(),
        None,
        AccountId::new("alice").unwrap(),
        Timestamp::from_unix_seconds(1_710_000_100),
        CommentBody::new("Ответ").unwrap(),
        Visibility::Public,
    )
    .unwrap()
    .hide();

    assert!(comment.is_hidden());
}

#[test]
fn post_can_be_edited() {
    let post = Post::new(
        PostId::new("post-1").unwrap(),
        ResourceId::new("blog-1").unwrap(),
        AccountId::new("alice").unwrap(),
        Timestamp::from_unix_seconds(1_710_000_000),
        PostBody::new("Первая запись").unwrap(),
        Visibility::Public,
    )
    .unwrap()
    .edit(PostBody::new("Исправленная запись").unwrap());

    assert_eq!(post.body().as_str(), "Исправленная запись");
}

#[test]
fn account_id_rejects_blank_value() {
    let error = AccountId::new(" ").expect_err("blank account id must be rejected");

    assert_eq!(error, DomainError::BlankIdentifier("account_id"));
}

#[test]
fn event_id_rejects_blank_value() {
    let error = EventId::new("").expect_err("blank event id must be rejected");

    assert_eq!(error, DomainError::BlankIdentifier("event_id"));
}

#[test]
fn timestamp_keeps_unix_seconds() {
    let timestamp = Timestamp::from_unix_seconds(1_710_000_000);

    assert_eq!(timestamp.as_unix_seconds(), 1_710_000_000);
}

#[test]
fn post_created_event_keeps_identity_and_payload() {
    let event = PostCreated::new(
        EventId::new("event-1").unwrap(),
        PostId::new("post-1").unwrap(),
        ResourceId::new("blog-1").unwrap(),
        AccountId::new("alice").unwrap(),
        Timestamp::from_unix_seconds(1_710_000_000),
        Visibility::Public,
    );

    assert_eq!(event.event_id().as_str(), "event-1");
    assert_eq!(event.post_id().as_str(), "post-1");
    assert_eq!(event.resource_id().as_str(), "blog-1");
    assert_eq!(event.actor_id().as_str(), "alice");
    assert_eq!(event.created_at().as_unix_seconds(), 1_710_000_000);
    assert_eq!(event.visibility(), Visibility::Public);
}

#[test]
fn comment_created_event_keeps_parent_comment_link() {
    let event = CommentCreated::new(
        EventId::new("event-2").unwrap(),
        CommentId::new("comment-1").unwrap(),
        PostId::new("post-1").unwrap(),
        Some(CommentId::new("comment-root").unwrap()),
        ResourceId::new("blog-1").unwrap(),
        AccountId::new("alice").unwrap(),
        Timestamp::from_unix_seconds(1_710_000_100),
        Visibility::FriendsOnly,
    );

    assert_eq!(event.event_id().as_str(), "event-2");
    assert_eq!(event.comment_id().as_str(), "comment-1");
    assert_eq!(event.post_id().as_str(), "post-1");
    assert_eq!(
        event.parent_comment_id().map(CommentId::as_str),
        Some("comment-root")
    );
    assert_eq!(event.resource_id().as_str(), "blog-1");
    assert_eq!(event.actor_id().as_str(), "alice");
    assert_eq!(event.created_at().as_unix_seconds(), 1_710_000_100);
    assert_eq!(event.visibility(), Visibility::FriendsOnly);
}

#[test]
fn post_hidden_event_keeps_identity_and_actor() {
    let event = liveletters_domain::PostHidden::new(
        EventId::new("event-3").unwrap(),
        PostId::new("post-1").unwrap(),
        ResourceId::new("blog-1").unwrap(),
        AccountId::new("alice").unwrap(),
        Timestamp::from_unix_seconds(1_710_000_200),
    );

    assert_eq!(event.event_id().as_str(), "event-3");
    assert_eq!(event.post_id().as_str(), "post-1");
    assert_eq!(event.resource_id().as_str(), "blog-1");
    assert_eq!(event.actor_id().as_str(), "alice");
    assert_eq!(event.created_at().as_unix_seconds(), 1_710_000_200);
}
