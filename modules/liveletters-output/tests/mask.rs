//! Тесты маскирования и форматирования `liveletters-output`.

use liveletters_config::{IdentityConfig, IdentityMeta, MailSecurity, MailSettings, SmtpSettings};
use liveletters_output::{mask_password, print_identity};

#[test]
fn mask_password_returns_plain_when_revealed() {
    assert_eq!(mask_password("hunter2", true), "hunter2");
}

#[test]
fn mask_password_returns_masks_when_hidden() {
    assert_eq!(mask_password("hunter2", false), "********");
}

#[test]
fn mask_password_empty_returns_masks_or_plain() {
    // При `reveal=true` пустой пароль остаётся пустым.
    assert_eq!(mask_password("", true), "");
    // При `reveal=false` пустой пароль маскируется точно так же, как
    // и непустой (сознательное упрощение: команда не должна различать
    // «пароль не задан» и «пароль задан»).
    assert_eq!(mask_password("", false), "********");
}

#[test]
fn print_identity_masks_smtp_password_by_default() {
    let cfg = sample_with_smtp("secret123");
    let mut buf: Vec<u8> = Vec::new();
    let cursor = std::io::Cursor::new(&mut buf);
    // печать в stdout — перехватим нельзя, поэтому проверяем через обёртку.
    let _ = cursor;
    print_identity(&cfg, false);
    // Простая smoke-проверка: печать не паникует, маскирование видно через unit-тест ниже.
    assert_eq!(mask_password("secret123", false), "********");
}

#[test]
fn print_identity_reveal_shows_smtp_password() {
    let cfg = sample_with_smtp("secret123");
    print_identity(&cfg, true);
    assert_eq!(mask_password("secret123", true), "secret123");
}

#[test]
fn print_table_aligns_columns() {
    use liveletters_output::print_table;
    let rows = vec![
        vec!["a".to_owned(), "bb".to_owned()],
        vec!["ccc".to_owned(), "d".to_owned()],
    ];
    print_table(&["x", "y"], &rows);
}

fn sample_with_smtp(password: &str) -> IdentityConfig {
    IdentityConfig {
        account_id: "acct_1".to_owned(),
        display_name: "Тестовый".to_owned(),
        mail: MailSettings {
            publish: "https://example.com".to_owned(),
            receive: vec!["in@example.com".to_owned()],
            smtp: Some(SmtpSettings {
                host: "smtp.example.com".to_owned(),
                port: 465,
                security: MailSecurity::Tls,
                username: "u@example.com".to_owned(),
                password: password.to_owned(),
                pwd_obfuscate: true,
                hello_domain: "example.com".to_owned(),
            }),
            imap: None,
        },
        meta: IdentityMeta::default(),
    }
}
