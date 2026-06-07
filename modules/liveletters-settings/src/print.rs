use liveletters_config::LogConfig;
use liveletters_output::{mask_password, print_kv};
use liveletters_store::{MailSettingsRecord, UserSettingsRecord};

pub fn print_settings(user: Option<&UserSettingsRecord>, mail: Option<&MailSettingsRecord>) {
    match user {
        None => println!(
            "[user_settings] отсутствует (запустите `lltt settings set …` или `lltt init --force`)"
        ),
        Some(u) => {
            println!("[user_settings]");
            print_kv(&[
                ("profile_id", &u.profile_id),
                ("nickname", &u.nickname),
                ("email_address", &u.email_address),
                ("avatar_url", u.avatar_url.as_deref().unwrap_or("")),
                ("language", &u.language),
                ("setup_completed", &u.setup_completed.to_string()),
            ]);
        }
    }
    println!();
    match mail {
        None => println!("[mail_settings] отсутствует"),
        Some(m) => {
            println!("[mail_settings]");
            print_kv(&[
                ("smtp.host", &m.smtp_host),
                ("smtp.port", &m.smtp_port.to_string()),
                ("smtp.security", &m.smtp_security),
                ("smtp.username", &m.smtp_username),
                ("smtp.password", &mask_password(&m.smtp_password, false)),
                ("smtp.hello_domain", &m.smtp_hello_domain),
                ("imap.host", &m.imap_host),
                ("imap.port", &m.imap_port.to_string()),
                ("imap.security", &m.imap_security),
                ("imap.username", &m.imap_username),
                ("imap.password", &mask_password(&m.imap_password, false)),
                ("imap.mailbox", &m.imap_mailbox),
            ]);
        }
    }
}

/// Печатает секцию журнала, только если хотя бы одно поле отличается от дефолта.
pub fn print_log_config(log: &LogConfig) {
    let defaults = LogConfig::default();
    if log == &defaults {
        return;
    }
    println!();
    println!("[логирование]");
    print_kv(&[
        ("log.destination", &log.destination.to_string()),
        ("log.level", &log.level.to_string()),
        ("log.max_size_bytes", &log.max_size_bytes.to_string()),
        ("log.keep_files", &log.keep_files.to_string()),
        ("log.include_bodies", &log.include_bodies.to_string()),
    ]);
    if log.level == liveletters_config::LogLevel::Off {
        println!(
            "(сейчас журнал выключен; используйте `lltt settings set log.level info` чтобы включить)"
        );
    }
}
