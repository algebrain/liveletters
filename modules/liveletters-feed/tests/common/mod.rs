//! Общие фикстуры для тестов `liveletters-feed`.

use liveletters_app_core::{HomeFeed, PostSummary};

pub fn sample_post(id: &str, body: &str, hidden: bool) -> PostSummary {
    PostSummary {
        post_id: id.to_owned(),
        resource_id: "blog-1".to_owned(),
        author_id: "alice".to_owned(),
        created_at: 0,
        body: body.to_owned(),
        visibility: "public".to_owned(),
        hidden,
    }
}

pub fn feed_with(posts: Vec<PostSummary>) -> HomeFeed {
    HomeFeed::new(posts)
}
