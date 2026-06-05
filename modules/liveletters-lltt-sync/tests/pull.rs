use liveletters_lltt_sync::{compute_next_cursor_uid, parse_security, tally};
use liveletters_mail::{MailSecurity, ReceivedEmail};

#[test]
fn parse_security_recognises_known_values() {
    assert!(matches!(
        parse_security("starttls"),
        Ok(MailSecurity::StartTls)
    ));
    assert!(matches!(parse_security("tls"), Ok(MailSecurity::Tls)));
    assert!(matches!(parse_security("none"), Ok(MailSecurity::None)));
    assert!(parse_security("foo").is_err());
}

#[test]
fn compute_next_cursor_uid_advances() {
    let emails = vec![
        received("imap-uid-11"),
        received("imap-uid-12"),
        received("imap-uid-14"),
    ];
    assert_eq!(compute_next_cursor_uid(10, &emails), 14);
}

#[test]
fn compute_next_cursor_uid_keeps_prev_on_empty() {
    let emails: Vec<ReceivedEmail> = vec![];
    assert_eq!(compute_next_cursor_uid(42, &emails), 42);
}

#[test]
fn compute_next_cursor_uid_ignores_malformed_ids() {
    let emails = vec![
        received("imap-uid-7"),
        received("garbage"),
        received("imap-uid-5"),
    ];
    assert_eq!(compute_next_cursor_uid(10, &emails), 10);
}

#[test]
fn tally_counts_outcome_kinds() {
    use liveletters_sync::SyncMessageOutcome;

    let report = liveletters_sync::SyncReport::new(vec![
        SyncMessageOutcome::Applied {
            message_id: "m1".into(),
            event_id: "e1".into(),
        },
        SyncMessageOutcome::Applied {
            message_id: "m2".into(),
            event_id: "e2".into(),
        },
        SyncMessageOutcome::Duplicate {
            message_id: "m3".into(),
            event_id: "e3".into(),
        },
        SyncMessageOutcome::Malformed {
            message_id: "m4".into(),
            reason: "bad".into(),
        },
        SyncMessageOutcome::Filtered {
            message_id: "m5".into(),
            event_id: "e5".into(),
            reason: "not subscribed".into(),
        },
    ]);

    let counts = tally(&report);
    assert_eq!(counts.applied, 2);
    assert_eq!(counts.duplicates, 1);
    assert_eq!(counts.malformed, 1);
}

fn received(message_id: &str) -> ReceivedEmail {
    ReceivedEmail {
        message_id: message_id.to_owned(),
        raw_message: String::new(),
    }
}
