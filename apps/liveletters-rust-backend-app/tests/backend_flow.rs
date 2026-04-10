use liveletters_rust_backend_app::{
    BackendApp, CreatePostRequest, SaveSettingsRequest,
};
use liveletters_store::{RawEventRecord, RawMessageRecord, Store};

#[test]
fn backend_wires_app_core_and_exposes_feed() {
    let backend = BackendApp::open_in_memory().expect("backend should open");

    backend
        .create_post(CreatePostRequest {
            post_id: "post-1",
            resource_id: "blog-1",
            author_id: "alice",
            created_at: 1,
            body: "First post",
        })
        .expect("create_post should work");

    let feed = backend.get_home_feed().expect("feed should load");

    assert_eq!(feed.posts().len(), 1);
    assert_eq!(feed.posts()[0].post_id, "post-1");
}

#[test]
fn backend_exposes_sync_status_and_failures_boundary() {
    let backend = BackendApp::open_in_memory().expect("backend should open");

    let status = backend.get_sync_status().expect("status should load");
    let failures = backend
        .list_incoming_failures()
        .expect("failures should load");

    assert_eq!(status.status, "healthy");
    assert!(failures.is_empty());
}

#[test]
fn backend_sync_status_reflects_richer_diagnostics_counters() {
    let store = Store::open_in_memory().expect("store should open");
    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "message-1".into(),
            raw_message: "unauthorized".into(),
            status: "unauthorized".into(),
        })
        .unwrap();
    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "message-2".into(),
            raw_message: "invalid".into(),
            status: "invalid".into(),
        })
        .unwrap();
    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "message-3".into(),
            raw_message: "replay".into(),
            status: "replay".into(),
        })
        .unwrap();

    let backend = BackendApp::from_store(store);
    let status = backend.get_sync_status().expect("status should load");

    assert_eq!(status.status, "degraded");
    assert_eq!(status.unauthorized_messages, 1);
    assert_eq!(status.invalid_messages, 1);
    assert_eq!(status.replayed_messages, 1);
}

#[test]
fn backend_failure_boundary_can_expose_raw_event_failures() {
    let store = Store::open_in_memory().expect("store should open");
    store
        .save_raw_event_record(&RawEventRecord {
            event_id: "event-1".into(),
            event_type: "comment_edited".into(),
            resource_id: "blog-1".into(),
            payload_json: "{\"kind\":\"comment_edited\"}".into(),
            apply_status: "unauthorized".into(),
            failure_reason: Some("actor_cannot_edit_comment".into()),
        })
        .unwrap();

    let backend = BackendApp::from_store(store);
    let failures = backend
        .list_event_failures()
        .expect("event failures should load");

    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].apply_status, "unauthorized");
    assert_eq!(
        failures[0].failure_reason.as_deref(),
        Some("actor_cannot_edit_comment")
    );
}

#[test]
fn backend_exposes_bootstrap_state_and_settings_boundary() {
    let backend = BackendApp::open_in_memory().expect("backend should open");

    let bootstrap = backend
        .get_bootstrap_state()
        .expect("bootstrap state should load");
    let settings = backend.get_settings().expect("settings should load");

    assert!(!bootstrap.setup_completed);
    assert_eq!(settings.nickname, "");
    assert_eq!(settings.smtp_port, 587);
    assert_eq!(settings.smtp_security, "starttls");
    assert_eq!(settings.imap_mailbox, "INBOX");
}

#[test]
fn backend_can_save_settings_through_app_core_boundary() {
    let backend = BackendApp::open_in_memory().expect("backend should open");

    let settings = backend
        .save_settings(SaveSettingsRequest {
            nickname: "alice",
            email_address: "alice@example.com",
            avatar_url: Some("https://example.com/avatar.png"),
            smtp_host: "smtp.example.com",
            smtp_port: 587,
            smtp_security: "starttls",
            smtp_username: "alice",
            smtp_password: "secret",
            smtp_hello_domain: "example.com",
            imap_host: "imap.example.com",
            imap_port: 143,
            imap_security: "starttls",
            imap_username: "alice",
            imap_password: "secret",
            imap_mailbox: "INBOX",
        })
        .expect("settings should save");

    assert!(settings.setup_completed);
    assert_eq!(settings.nickname, "alice");

    let bootstrap = backend
        .get_bootstrap_state()
        .expect("bootstrap state should load");
    let loaded = backend.get_settings().expect("settings should load");

    assert!(bootstrap.setup_completed);
    assert_eq!(loaded.email_address, "alice@example.com");
    assert_eq!(loaded.smtp_host, "smtp.example.com");
    assert_eq!(loaded.imap_security, "starttls");
}

#[test]
fn backend_can_save_settings_without_explicit_smtp_hello_domain() {
    let backend = BackendApp::open_in_memory().expect("backend should open");

    let settings = backend
        .save_settings(SaveSettingsRequest {
            nickname: "alice",
            email_address: "alice@example.com",
            avatar_url: None,
            smtp_host: "smtp.example.com",
            smtp_port: 587,
            smtp_security: "starttls",
            smtp_username: "alice",
            smtp_password: "secret",
            smtp_hello_domain: "",
            imap_host: "imap.example.com",
            imap_port: 143,
            imap_security: "starttls",
            imap_username: "alice",
            imap_password: "secret",
            imap_mailbox: "INBOX",
        })
        .expect("settings should save with inferred hello domain");

    assert_eq!(settings.smtp_hello_domain, "example.com");
}
