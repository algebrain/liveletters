use liveletters_utils::email::{email_local_part, looks_like_email};

#[test]
fn accepts_simple_email_with_local_and_domain_parts() {
    assert!(looks_like_email("alice@example.org"));
}

#[test]
fn trims_outer_whitespace_before_checking_email_shape() {
    assert!(looks_like_email("  alice@example.org  "));
    assert_eq!(email_local_part("  alice@example.org  "), Some("alice"));
}

#[test]
fn rejects_missing_or_empty_email_parts() {
    for value in ["alice", "@example.org", "alice@", "alice@@example.org"] {
        assert!(!looks_like_email(value), "{value:?} should be rejected");
        assert_eq!(email_local_part(value), None);
    }
}

#[test]
fn rejects_email_with_internal_whitespace() {
    assert!(!looks_like_email("alice @example.org"));
    assert!(!looks_like_email("alice@example .org"));
}

#[test]
fn returns_local_part_without_allocating_or_trimming_inside_it() {
    assert_eq!(
        email_local_part("first.last@example.org"),
        Some("first.last")
    );
}
