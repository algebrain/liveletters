use liveletters_config::{IdentityConfig, MailSecurity};
use liveletters_domain::ResourceAddress;

const MINIMAL_TOML: &str = r#"
account_id = "acct_alice_3kf"
display_name = "Alice"

[mail]
publish = "alice-publish@example.org"
receive = ["alice-feed@example.org"]
"#;

const FULL_TOML: &str = r#"
account_id = "acct_bob_9xz"
display_name = "Bob"

[mail]
publish = "bob-publish@example.org"
receive = ["bob-feed@example.org", "bob-feed2@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "bob"
password = "smtp-secret"
pwd_obfuscate = true
hello_domain = "example.org"

[mail.imap]
host = "imap.example.org"
port = 143
security = "tls"
username = "bob"
password = "imap-secret"
pwd_obfuscate = true

[meta]
resources_owned = ["blog-1", "blog-2"]
subscriptions = ["alice-publish@example.org", "carol-publish@example.org"]
"#;

#[test]
fn parses_minimal_identity_toml() {
    let cfg: IdentityConfig = toml::from_str(MINIMAL_TOML).unwrap();
    assert_eq!(cfg.account_id(), "acct_alice_3kf");
    assert_eq!(cfg.display_name(), "Alice");
    assert_eq!(cfg.mail().publish(), "alice-publish@example.org");
    assert_eq!(cfg.mail().receive(), &["alice-feed@example.org"]);
    assert!(cfg.mail().smtp().is_none());
    assert!(cfg.mail().imap().is_none());
    assert!(cfg.resources_owned().is_empty());
    assert!(cfg.subscriptions().is_empty());
}

#[test]
fn parses_full_identity_toml_with_subscriptions() {
    let cfg: IdentityConfig = toml::from_str(FULL_TOML).unwrap();
    assert_eq!(cfg.account_id(), "acct_bob_9xz");
    assert_eq!(cfg.display_name(), "Bob");
    assert_eq!(cfg.mail().publish(), "bob-publish@example.org");
    assert_eq!(
        cfg.mail().receive(),
        &["bob-feed@example.org", "bob-feed2@example.org"]
    );

    let smtp = cfg.mail().smtp().expect("smtp should be present");
    assert_eq!(smtp.host(), "smtp.example.org");
    assert_eq!(smtp.port(), 587);
    assert_eq!(smtp.security(), MailSecurity::StartTls);
    assert_eq!(smtp.username(), "bob");
    assert_eq!(smtp.password(), "smtp-secret");
    assert!(smtp.pwd_obfuscate());
    assert_eq!(smtp.hello_domain(), "example.org");

    let imap = cfg.mail().imap().expect("imap should be present");
    assert_eq!(imap.host(), "imap.example.org");
    assert_eq!(imap.port(), 143);
    assert_eq!(imap.security(), MailSecurity::Tls);
    assert_eq!(imap.username(), "bob");
    assert_eq!(imap.password(), "imap-secret");
    assert!(imap.pwd_obfuscate());

    assert_eq!(cfg.resources_owned(), &["blog-1", "blog-2"]);
    assert_eq!(
        cfg.subscriptions(),
        &[
            ResourceAddress::new("alice-publish@example.org").unwrap(),
            ResourceAddress::new("carol-publish@example.org").unwrap(),
        ]
    );
}

#[test]
fn pwd_obfuscate_defaults_to_true_when_omitted() {
    let cfg: IdentityConfig = toml::from_str(
        r#"
account_id = "acct_alice"
display_name = "Alice"

[mail]
publish = "alice@example.org"

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "alice"
password = "secret"

[mail.imap]
host = "imap.example.org"
port = 993
security = "tls"
username = "alice"
password = "secret"
"#,
    )
    .unwrap();

    assert!(cfg.mail().smtp().unwrap().pwd_obfuscate());
    assert!(cfg.mail().imap().unwrap().pwd_obfuscate());
}

#[test]
fn parse_rejects_missing_required_fields() {
    let toml = r#"
        display_name = "Alice"
        [mail]
        publish = "alice@example.org"
        receive = []
    "#;
    let err = toml::from_str::<IdentityConfig>(toml).unwrap_err();
    let _ = err;
}

#[test]
fn parse_accepts_default_meta_when_omitted() {
    let toml = r#"
        account_id = "acct_x"
        display_name = "X"
        [mail]
        publish = "x@example.org"
    "#;
    let cfg: IdentityConfig = toml::from_str(toml).unwrap();
    assert!(cfg.resources_owned().is_empty());
    assert!(cfg.subscriptions().is_empty());
    assert!(cfg.mail().receive().is_empty());
    assert!(cfg.mail().smtp().is_none());
}
