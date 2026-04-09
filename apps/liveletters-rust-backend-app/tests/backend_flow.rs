use liveletters_rust_backend_app::{BackendApp, CreatePostRequest};

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
