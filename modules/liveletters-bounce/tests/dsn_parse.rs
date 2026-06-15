//! Распознавание и парсинг DSN-bounce (Delivery Status Notification).

use liveletters_bounce::{BounceAction, parse_dsn};

const DSN_FIXTURE: &str = "From: MAILER-DAEMON@yandex.ru\r\n\
To: alice@yandex.ru\r\n\
Subject: Mail delivery failed\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/report; report-type=delivery-status;\r\n\
 boundary=\"dsn-boundary\"\r\n\
\r\n\
--dsn-boundary\r\n\
Content-Description: Notification\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
This is the mail system at host yandex.ru.\r\n\
\r\n\
--dsn-boundary\r\n\
Content-Description: Delivery status\r\n\
Content-Type: message/delivery-status\r\n\
\r\n\
Reporting-MTA: dns; yandex.ru\r\n\
Action: failed\r\n\
Status: 5.1.1\r\n\
Final-Recipient: rfc822; nobody@example.org\r\n\
Diagnostic-Code: smtp; 550 5.1.1 User unknown\r\n\
Original-Message-ID: <subscription-requested:bob@example.org:1700000000@yandex.ru>\r\n\
\r\n\
--dsn-boundary\r\n\
Content-Description: Original message\r\n\
Content-Type: message/rfc822\r\n\
\r\n\
From: alice@yandex.ru\r\n\
To: nobody@example.org\r\n\
Subject: Запрос подписки\r\n\
\r\n\
--dsn-boundary--\r\n";

const ARF_FIXTURE: &str = "From: complaints@isp.example\r\n\
To: abuse-report@example.com\r\n\
Subject: Abuse report\r\n\
Content-Type: multipart/report; report-type=feedback;\r\n\
\r\n\
Это ARF (RFC 5965), а не DSN. Не путать.\r\n";

const PLAIN_FIXTURE: &str = "From: someone@example.org\r\n\
To: me@example.org\r\n\
Subject: Hello\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Hello world.\r\n";

#[test]
fn parses_standard_dsn_5_1_1() {
    let report = parse_dsn(DSN_FIXTURE).unwrap().expect("должен быть DSN");
    assert_eq!(report.action, BounceAction::Failed);
    assert_eq!(report.status, "5.1.1");
    assert_eq!(report.final_recipient, "nobody@example.org");
    assert!(report.diagnostic_code.contains("User unknown"));
    assert_eq!(
        report.original_message_id.as_deref(),
        Some("subscription-requested:bob@example.org:1700000000@yandex.ru")
    );
}

#[test]
fn ignores_arf_feedback_report() {
    let report = parse_dsn(ARF_FIXTURE).unwrap();
    assert!(report.is_none(), "ARF — не DSN");
}

#[test]
fn ignores_plain_email() {
    let report = parse_dsn(PLAIN_FIXTURE).unwrap();
    assert!(report.is_none(), "обычное письмо — не DSN");
}
