//! Функции человекочитаемого вывода для команд `lltt`.
//!
//! Все функции пишут в `stdout` без цветов и без зависимостей от терминала;
//! маскирование секретов управляется флагом `reveal`.

/// Возвращает открытый пароль, если `reveal=true`, иначе маскированную строку.
///
/// **Семантика `mask_password(_, false)`:** всегда возвращает `********`,
/// в том числе для пустого пароля. Это сознательное упрощение: команда,
/// печатающая пароль, не должна различать «пусто» и «непусто» (иначе
/// раскрывается факт установки секрета). Чтобы узнать, задан ли пароль
/// на самом деле, обратитесь к БД напрямую (`SELECT smtp_password
/// FROM mail_settings`) или используйте приватный API крейта
/// `liveletters-store`.
pub fn mask_password(plain: &str, reveal: bool) -> String {
    if reveal {
        plain.to_owned()
    } else {
        "********".to_owned()
    }
}

/// Печатает пары `key: value` построчно.
pub fn print_kv(pairs: &[(&str, &str)]) {
    for (k, v) in pairs {
        println!("{k}: {v}");
    }
}

/// Печатает таблицу с заголовками и строками; колонки выравниваются пробелами.
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i >= widths.len() {
                widths.push(0);
            }
            let w = cell.chars().count();
            if w > widths[i] {
                widths[i] = w;
            }
        }
    }
    print_row(headers, &widths);
    println!();
    for row in rows {
        let cells: Vec<&str> = row.iter().map(String::as_str).collect();
        print_row(&cells, &widths);
        println!();
    }
}

fn print_row(cells: &[&str], widths: &[usize]) {
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            print!("  ");
        }
        let pad = widths
            .get(i)
            .copied()
            .unwrap_or(0)
            .saturating_sub(cell.chars().count());
        print!("{cell}{}", " ".repeat(pad));
    }
}

/// Печатает идентичность в человекочитаемом виде; пароли маскируются без `reveal`.
pub fn print_identity(cfg: &liveletters_config::IdentityConfig, reveal: bool) {
    println!("[identity]");
    print_kv(&[
        ("account_id", cfg.account_id()),
        ("display_name", cfg.display_name()),
    ]);

    println!();
    println!("[mail]");
    print_kv(&[("publish", cfg.mail().publish())]);
    let receive = cfg.mail().receive();
    if receive.is_empty() {
        print_kv(&[("receive", "-")]);
    } else {
        for (i, r) in receive.iter().enumerate() {
            print_kv(&[("receive", &format!("[{i}] {r}"))]);
        }
    }

    if let Some(smtp) = cfg.mail().smtp() {
        println!();
        println!("[mail.smtp]");
        print_kv(&[
            ("host", smtp.host()),
            ("port", &smtp.port().to_string()),
            ("security", smtp.security().as_str()),
            ("username", smtp.username()),
            ("password", &mask_password(smtp.password(), reveal)),
        ]);
    }

    if let Some(imap) = cfg.mail().imap() {
        println!();
        println!("[mail.imap]");
        print_kv(&[
            ("host", imap.host()),
            ("port", &imap.port().to_string()),
            ("security", imap.security().as_str()),
            ("username", imap.username()),
            ("password", &mask_password(imap.password(), reveal)),
            ("mailbox", imap.mailbox()),
        ]);
    }

    if !cfg.resources_owned().is_empty() {
        println!();
        println!("[resources_owned]");
        for r in cfg.resources_owned() {
            println!("- {r}");
        }
    }
}

pub fn print_identity_from_db(
    name: &str,
    user: &liveletters_store::UserSettingsRecord,
    mail: Option<&liveletters_store::MailSettingsRecord>,
    receive: &[String],
    resources: &[String],
    local_subs: &[String],
    reveal: bool,
) {
    println!("[identity]");
    print_kv(&[
        ("account_id", &format!("acct_{name}")),
        ("display_name", &user.nickname),
        ("language", &user.language),
    ]);

    println!();
    println!("[mail]");
    print_kv(&[("publish", &user.email_address)]);
    if receive.is_empty() {
        print_kv(&[("receive", "-")]);
    } else {
        for (i, r) in receive.iter().enumerate() {
            let label = format!("[{i}] {r}");
            let kv: [(&str, &str); 1] = [("receive", &label)];
            print_kv(&kv);
        }
    }

    if let Some(mail) = mail {
        if !mail.smtp_host.is_empty() {
            println!();
            println!("[mail.smtp]");
            print_kv(&[
                ("host", &mail.smtp_host),
                ("port", &mail.smtp_port.to_string()),
                ("security", &mail.smtp_security),
                ("username", &mail.smtp_username),
                ("password", &mask_password(&mail.smtp_password, reveal)),
            ]);
        }
        if !mail.imap_host.is_empty() {
            println!();
            println!("[mail.imap]");
            print_kv(&[
                ("host", &mail.imap_host),
                ("port", &mail.imap_port.to_string()),
                ("security", &mail.imap_security),
                ("username", &mail.imap_username),
                ("password", &mask_password(&mail.imap_password, reveal)),
                ("mailbox", &mail.imap_mailbox),
            ]);
        }
    }

    if !resources.is_empty() {
        println!();
        println!("[resources_owned]");
        for r in resources {
            println!("- {r}");
        }
    }

    if !local_subs.is_empty() {
        println!();
        println!("[local_subscriptions]");
        for s in local_subs {
            println!("- {s}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_password_empty_returns_eight_stars() {
        assert_eq!(mask_password("", false), "********");
    }

    #[test]
    fn mask_password_non_empty_returns_eight_stars() {
        assert_eq!(mask_password("hunter2", false), "********");
    }

    #[test]
    fn mask_password_reveal_returns_plain() {
        assert_eq!(mask_password("hunter2", true), "hunter2");
    }

    #[test]
    fn mask_password_reveal_empty_returns_empty() {
        assert_eq!(mask_password("", true), "");
    }
}
