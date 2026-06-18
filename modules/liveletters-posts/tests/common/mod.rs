//! Общие фикстуры для тестов `liveletters-posts`.

use liveletters_app_core::{CurrentUserPosts, PostSummary};

pub fn sample_post(id: &str, body: &str, hidden: bool) -> PostSummary {
    PostSummary {
        post_id: id.to_owned(),
        resource_id: "blog-1".to_owned(),
        author_id: "alice".to_owned(),
        author_display: "Alice <alice@example.org>".to_owned(),
        created_at: 0,
        body: body.to_owned(),
        visibility: "public".to_owned(),
        hidden,
    }
}

pub fn posts_with(posts: Vec<PostSummary>) -> CurrentUserPosts {
    CurrentUserPosts::new(posts)
}
