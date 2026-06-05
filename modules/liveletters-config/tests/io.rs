use liveletters_config::{
    GlobalConfig, IdentityConfig, ImapSettings, MailSecurity, SmtpSettings, list_identities,
    load_global, load_identity, map_identity_to_settings, save_identity, settings_to_identity,
};
use liveletters_domain::ResourceAddress;

fn sample_identity() -> IdentityConfig {
    IdentityConfig {
        account_id: "acct_alice_3kf".into(),
        display_name: "Alice".into(),
        mail: liveletters_config::MailSettings {
            publish: "alice-publish@example.org".into(),
            receive: vec!["alice-feed@example.org".into()],
            smtp: Some(SmtpSettings {
                host: "smtp.example.org".into(),
                port: 587,
                security: MailSecurity::StartTls,
                username: "alice".into(),
                password: "smtp-secret".into(),
                pwd_obfuscate: true,
                hello_domain: "example.org".into(),
            }),
            imap: Some(ImapSettings {
                host: "imap.example.org".into(),
                port: 143,
                security: MailSecurity::Tls,
                username: "alice".into(),
                password: "imap-secret".into(),
                pwd_obfuscate: true,
                mailbox: "INBOX".into(),
            }),
        },
        meta: liveletters_config::IdentityMeta {
            resources_owned: vec!["blog-1".into()],
            subscriptions: vec![ResourceAddress::new("bob-publish@example.org").unwrap()],
        },
    }
}

#[test]
fn save_and_load_identity_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = sample_identity();

    save_identity(tmp.path(), "alice", &cfg).expect("identity should be saved");
    let loaded = load_identity(tmp.path(), "alice").expect("identity should be loaded");

    assert_eq!(loaded.account_id(), cfg.account_id());
    assert_eq!(loaded.display_name(), cfg.display_name());
    assert_eq!(loaded.mail().publish(), cfg.mail().publish());
    assert_eq!(loaded.mail().receive(), cfg.mail().receive());
    assert_eq!(
        loaded.mail().smtp().map(|s| s.host()),
        cfg.mail().smtp().map(|s| s.host()),
    );
    assert_eq!(loaded.resources_owned(), cfg.resources_owned());
    assert_eq!(loaded.subscriptions(), cfg.subscriptions());
    assert!(
        list_identities(tmp.path())
            .unwrap()
            .contains(&"alice".to_string())
    );
}

#[test]
fn list_identities_returns_empty_when_dir_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let names = list_identities(tmp.path()).unwrap();
    assert!(names.is_empty());
}

#[test]
fn list_identities_returns_all_saved_names_sorted() {
    let tmp = tempfile::tempdir().unwrap();
    save_identity(tmp.path(), "charlie", &sample_identity()).unwrap();
    save_identity(tmp.path(), "alice", &sample_identity()).unwrap();
    save_identity(tmp.path(), "bob", &sample_identity()).unwrap();

    let names = list_identities(tmp.path()).unwrap();
    assert_eq!(names, vec!["alice", "bob", "charlie"]);
}

#[test]
fn load_identity_returns_unknown_when_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let err = load_identity(tmp.path(), "ghost").unwrap_err();
    assert!(
        matches!(err, liveletters_config::ConfigError::UnknownIdentity(name) if name == "ghost")
    );
}

#[test]
fn load_global_returns_default_when_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = load_global(tmp.path()).unwrap();
    assert_eq!(cfg, GlobalConfig::default());
}

#[test]
fn identity_settings_round_trip_via_app_settings() {
    let original = sample_identity();
    let settings = map_identity_to_settings(&original);
    let reconstructed = settings_to_identity(original.account_id(), &settings);

    assert_eq!(reconstructed.account_id(), original.account_id());
    assert_eq!(reconstructed.display_name(), original.display_name());
    assert_eq!(reconstructed.mail().publish(), original.mail().publish());
    assert_eq!(
        reconstructed
            .mail()
            .smtp()
            .map(|s| (s.host(), s.port(), s.security())),
        original
            .mail()
            .smtp()
            .map(|s| (s.host(), s.port(), s.security())),
    );
    assert_eq!(
        reconstructed.mail().imap().map(|s| s.mailbox()),
        original.mail().imap().map(|s| s.mailbox()),
    );
}
