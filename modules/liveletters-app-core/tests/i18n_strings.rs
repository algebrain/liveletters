//! Тесты модуля `i18n_strings`: проверяют, что subject/body
//! для всех событий переключаются по языку пользователя.

use liveletters_app_core::{
    AppCore, CreateCommentCommand, CreatePostCommand, EditCommentCommand, HidePostCommand,
    SubscribeCommand, UnsubscribeCommand, Visibility,
};
use liveletters_store::Store;
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn save_user(store: &Store, language: &str) {
    store
        .save_identity(
            "default",
            "alice@example.org",
            "alice",
            None,
            language,
            true,
        )
        .unwrap();
    for (email, nickname) in [
        ("blog-1", "blog"),
        ("alice", "alice"),
        ("Алиса", "Алиса"),
        ("alice-publish@example.org", "Алиса"),
        ("bob-feed@example.org", "Боб"),
    ] {
        store.save_author(email, nickname, "test").unwrap();
    }
}

fn outbox_subject_and_body(store: &Store, event_id: &str) -> (String, String) {
    // Локализованный subject хранится в `OutboxRecord.subject`
    // (он же попадает в SMTP-заголовок Subject). Локализованное тело
    // хранится в `OutboxRecord.human_readable_body` (он попадает в
    // text/plain под-часть при сборке письма). В JSON его нет —
    // (см. message.rs: skip_serializing default), потому что тело уже
    // есть в отдельной колонке outbox и не должно дублироваться в wire-формате.
    let records = store.list_outbox_records().unwrap();
    let record = records
        .iter()
        .find(|r| r.event_id == event_id)
        .expect("outbox record for event_id");
    (
        record.subject.clone().unwrap_or_default(),
        record.human_readable_body.clone().unwrap_or_default(),
    )
}

#[test]
fn post_created_subject_switches_with_language() {
    let (_dir_en, store_en) = open();
    save_user(&store_en, "en");
    let app_en = AppCore::new(&store_en);
    app_en
        .create_post(CreatePostCommand {
            profile_id: "default",
            post_id: "post-1",
            resource_id: "blog-1",
            author_id: "alice",
            created_at: 1,
            body: "Hello",
            visibility: Visibility::Public,
        })
        .unwrap();
    let (subject_en, body_en) = outbox_subject_and_body(&store_en, "post-created:post-1");
    assert!(
        subject_en.contains("New post in journal"),
        "subject={subject_en}"
    );
    assert!(body_en.contains("New post in journal"), "body={body_en}");

    let (_dir_ru, store_ru) = open();
    save_user(&store_ru, "ru");
    let app_ru = AppCore::new(&store_ru);
    app_ru
        .create_post(CreatePostCommand {
            profile_id: "default",
            post_id: "post-2",
            resource_id: "blog-1",
            author_id: "Алиса",
            created_at: 1,
            body: "Привет",
            visibility: Visibility::Public,
        })
        .unwrap();
    let (subject_ru, body_ru) = outbox_subject_and_body(&store_ru, "post-created:post-2");
    assert!(
        subject_ru.contains("Новая запись в журнале"),
        "subject_ru={subject_ru}"
    );
    assert!(
        body_ru.contains("Новая запись в журнале"),
        "body_ru={body_ru}"
    );
}

#[test]
fn comment_created_subject_uses_localized_template() {
    let (_dir, store) = open();
    save_user(&store, "ru");
    let app = AppCore::new(&store);
    app.create_post(CreatePostCommand {
        profile_id: "default",
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "alice",
        created_at: 1,
        body: "Тема",
        visibility: Visibility::Public,
    })
    .unwrap();
    app.create_comment(CreateCommentCommand {
        profile_id: "default",
        comment_id: "comment-1",
        post_id: "post-1",
        parent_comment_id: None,
        author_id: "alice",
        created_at: 2,
        body: "Первый",
    })
    .unwrap();

    let (subject, body) = outbox_subject_and_body(&store, "comment-created:comment-1");
    assert!(
        subject.contains("Новый комментарий от alice"),
        "subject={subject}"
    );
    assert!(body.contains("оставил(а) комментарий"), "body={body}");
}

#[test]
fn comment_edited_subject_uses_localized_template() {
    let (_dir, store) = open();
    save_user(&store, "en");
    let app = AppCore::new(&store);
    app.create_post(CreatePostCommand {
        profile_id: "default",
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "alice",
        created_at: 1,
        body: "Body",
        visibility: Visibility::Public,
    })
    .unwrap();
    app.create_comment(CreateCommentCommand {
        profile_id: "default",
        comment_id: "comment-1",
        post_id: "post-1",
        parent_comment_id: None,
        author_id: "alice",
        created_at: 2,
        body: "Original",
    })
    .unwrap();
    app.edit_comment(EditCommentCommand {
        profile_id: "default",
        comment_id: "comment-1",
        actor_id: "alice",
        created_at: 3,
        body: "Edited",
    })
    .unwrap();

    let (subject, _body) = outbox_subject_and_body(&store, "comment-edited:comment-1");
    assert!(
        subject.contains("Comment edited: alice"),
        "subject={subject}"
    );
}

#[test]
fn post_hidden_subject_uses_localized_template() {
    let (_dir, store) = open();
    save_user(&store, "ru");
    let app = AppCore::new(&store);
    app.create_post(CreatePostCommand {
        profile_id: "default",
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "alice",
        created_at: 1,
        body: "Тело",
        visibility: Visibility::Public,
    })
    .unwrap();
    app.hide_post(HidePostCommand {
        profile_id: "default",
        post_id: "post-1",
        actor_id: "alice",
        created_at: 2,
    })
    .unwrap();

    let (subject, body) = outbox_subject_and_body(&store, "post-hidden:post-1");
    assert!(
        subject.contains("Запись скрыта: alice"),
        "subject={subject}"
    );
    assert!(body.contains("скрыл(а) запись post-1"), "body={body}");
}

#[test]
fn subscription_requested_subject_uses_localized_template_en() {
    let (_dir_en, store_en) = open();
    save_user(&store_en, "en");
    let app_en = AppCore::new(&store_en);
    app_en
        .subscribe(SubscribeCommand {
            profile_id: "default",
            resource_address: "alice-publish@example.org",
            subscriber_delivery_address: "bob-feed@example.org",
            created_at: 1,
        })
        .unwrap();

    let records = store_en.list_outbox_records().unwrap();
    let sub = records
        .iter()
        .find(|r| r.event_id.starts_with("subscription:"))
        .expect("subscribe outbox row");
    assert!(
        sub.subject
            .as_deref()
            .unwrap_or("")
            .contains("New subscription: bob-feed@example.org"),
        "subject={:?}",
        sub.subject
    );
}

#[test]
fn subscription_requested_subject_uses_localized_template_ru() {
    let (_dir, store) = open();
    save_user(&store, "ru");
    let app = AppCore::new(&store);
    app.subscribe(SubscribeCommand {
        profile_id: "default",
        resource_address: "alice-publish@example.org",
        subscriber_delivery_address: "bob-feed@example.org",
        created_at: 1,
    })
    .unwrap();

    let records = store.list_outbox_records().unwrap();
    let sub = records
        .iter()
        .find(|r| r.event_id.starts_with("subscription:"))
        .expect("subscribe outbox row");
    assert!(
        sub.subject
            .as_deref()
            .unwrap_or("")
            .contains("Подписка: bob-feed@example.org"),
        "subject={:?}",
        sub.subject
    );
}

#[test]
fn subscription_revoked_subject_uses_localized_template_en() {
    let (_dir, store) = open();
    save_user(&store, "en");
    let app = AppCore::new(&store);
    app.subscribe(SubscribeCommand {
        profile_id: "default",
        resource_address: "alice-publish@example.org",
        subscriber_delivery_address: "bob-feed@example.org",
        created_at: 1,
    })
    .unwrap();
    app.unsubscribe(UnsubscribeCommand {
        profile_id: "default",
        resource_address: "alice-publish@example.org",
        subscriber_delivery_address: "bob-feed@example.org",
        created_at: 2,
    })
    .unwrap();

    let records = store.list_outbox_records().unwrap();
    let unsub = records
        .iter()
        .find(|r| r.event_id.starts_with("unsubscription:"))
        .expect("unsubscribe outbox row");
    assert!(
        unsub
            .subject
            .as_deref()
            .unwrap_or("")
            .contains("Unsubscribed: bob-feed@example.org"),
        "subject={:?}",
        unsub.subject
    );
}

#[test]
fn subscription_revoked_subject_uses_localized_template_ru() {
    let (_dir, store) = open();
    save_user(&store, "ru");
    let app = AppCore::new(&store);
    app.subscribe(SubscribeCommand {
        profile_id: "default",
        resource_address: "alice-publish@example.org",
        subscriber_delivery_address: "bob-feed@example.org",
        created_at: 1,
    })
    .unwrap();
    app.unsubscribe(UnsubscribeCommand {
        profile_id: "default",
        resource_address: "alice-publish@example.org",
        subscriber_delivery_address: "bob-feed@example.org",
        created_at: 2,
    })
    .unwrap();

    let records = store.list_outbox_records().unwrap();
    let unsub = records
        .iter()
        .find(|r| r.event_id.starts_with("unsubscription:"))
        .expect("unsubscribe outbox row");
    assert!(
        unsub
            .subject
            .as_deref()
            .unwrap_or("")
            .contains("Отписка: bob-feed@example.org"),
        "subject={:?}",
        unsub.subject
    );
}
