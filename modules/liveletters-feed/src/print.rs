use liveletters_app_core::{HomeFeed, PostSummary};
use liveletters_output::format_unix_iso8601_utc;
use liveletters_output::print_kv;

const BODY_MAX: usize = 80;

pub fn print_feed(feed: &HomeFeed, identity_display: &str, limit: Option<usize>) {
    let posts = feed.posts();
    let total = posts.len();
    let shown = match limit {
        Some(n) => n.min(total),
        None => total,
    };

    print_kv(&[("лента пользователя", identity_display)]);
    println!("постов: {total} (показано: {shown})");
    println!();

    if posts.is_empty() {
        println!("(пусто)");
        return;
    }

    for post in posts.iter().take(shown) {
        print_post(post);
    }
}

fn print_post(post: &PostSummary) {
    let visibility = if post.visibility.is_empty() {
        "—"
    } else {
        post.visibility.as_str()
    };
    let hidden_marker = if post.hidden { " (скрыт)" } else { "" };
    let created = format_unix_iso8601_utc(post.created_at);
    println!(
        "┌─ пост #{} от {}{hidden_marker}",
        post.post_id, post.author_id
    );
    println!("│  visibility: {visibility}");
    println!("│  {created}");
    let body = truncate_body(&post.body, BODY_MAX);
    for line in body.lines() {
        println!("│  {line}");
    }
    println!("└─");
    println!();
}

fn truncate_body(body: &str, max: usize) -> String {
    if body.chars().count() > max {
        let truncated: String = body.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    } else {
        body.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_body_short_keeps_as_is() {
        let result = truncate_body("hi", 80);
        assert_eq!(result, "hi");
    }

    #[test]
    fn truncate_body_long_adds_ellipsis() {
        let long: String = "x".repeat(120);
        let result = truncate_body(&long, 80);
        assert_eq!(result.chars().count(), 80);
        assert!(result.ends_with('…'));
    }
}
