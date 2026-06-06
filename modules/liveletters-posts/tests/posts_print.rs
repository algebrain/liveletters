//! Тесты `print_posts`: печать `CurrentUserPosts` в человекочитаемом виде.

mod common;

use liveletters_posts::print_posts;

#[test]
fn empty_posts_prints_empty_marker() {
    let posts = common::posts_with(vec![]);
    print_posts(&posts, "alice", None);
}

#[test]
fn single_post_prints_header_and_body() {
    let posts = common::posts_with(vec![common::sample_post("post_1", "Привет, мир", false)]);
    print_posts(&posts, "alice", None);
}

#[test]
fn limit_truncates_output() {
    let posts = common::posts_with(vec![
        common::sample_post("p1", "один", false),
        common::sample_post("p2", "два", false),
        common::sample_post("p3", "три", false),
    ]);
    print_posts(&posts, "alice", Some(1));
}

#[test]
fn hidden_post_shows_marker() {
    let posts = common::posts_with(vec![common::sample_post("p1", "секрет", true)]);
    print_posts(&posts, "alice", None);
}
