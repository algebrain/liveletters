//! Тесты `print_feed`: печать `HomeFeed` в человекочитаемом виде.

mod common;

use liveletters_feed::print_feed;

#[test]
fn empty_feed_prints_empty_marker() {
    let feed = common::feed_with(vec![]);
    print_feed(&feed, "alice", None);
}

#[test]
fn single_post_prints_header_and_body() {
    let feed = common::feed_with(vec![common::sample_post("post_1", "Привет, мир", false)]);
    print_feed(&feed, "alice", None);
}

#[test]
fn limit_truncates_output() {
    let feed = common::feed_with(vec![
        common::sample_post("p1", "один", false),
        common::sample_post("p2", "два", false),
        common::sample_post("p3", "три", false),
    ]);
    print_feed(&feed, "alice", Some(1));
}

#[test]
fn hidden_post_shows_marker() {
    let feed = common::feed_with(vec![common::sample_post("p1", "секрет", true)]);
    print_feed(&feed, "alice", None);
}
