use liveletters_output::format_unix_iso8601_utc;
use liveletters_output::print_kv;
use liveletters_store::PostRecord;

const BODY_MAX: usize = 80;

#[derive(Debug, Clone)]
pub struct FeedPost {
    record: PostRecord,
    author_display: String,
}

impl FeedPost {
    pub fn new(record: PostRecord, author_display: String) -> Self {
        Self {
            record,
            author_display,
        }
    }
}

pub fn print_feed(posts: &[FeedPost], identity_display: &str, limit: Option<usize>) {
    let total = posts.len();
    let shown = match limit {
        Some(n) => n.min(total),
        None => total,
    };

    print_kv(&[("лента подписок", identity_display)]);
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

fn print_post(post: &FeedPost) {
    let record = &post.record;
    let visibility = if record.visibility.is_empty() {
        "—"
    } else {
        record.visibility.as_str()
    };
    let hidden_marker = if record.hidden { " (скрыт)" } else { "" };
    let created = format_unix_iso8601_utc(record.created_at);
    println!(
        "┌─ пост #{} от {}{hidden_marker}",
        record.post_id, post.author_display
    );
    println!("│  visibility: {visibility}");
    println!("│  {created}");
    let body = truncate_body(&record.body, BODY_MAX);
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
