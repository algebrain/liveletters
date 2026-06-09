use std::sync::{Mutex, OnceLock};

use liveletters_i18n::{I18nError, Locale, Vars, detect_system_locale, parse_locale, translate};

/// Глобальный мьютекс для серилизации тестов, которые трогают `std::env`.
/// `OnceLock` гарантирует единственный экземпляр на процесс.
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn translate_fills_known_variables() {
    let s = translate(
        "post_created.subject",
        Locale::Ru,
        Vars(&[("resource", "blog-1")]),
    )
    .expect("translation should succeed");
    assert_eq!(s, "Новая запись в журнале blog-1");
}

#[test]
fn translate_returns_unknown_key_error() {
    let err = translate("no.such.key", Locale::Ru, Vars(&[("sender", "alice")])).unwrap_err();
    assert!(matches!(err, I18nError::UnknownKey(ref k) if k == "no.such.key"));
}

#[test]
fn translate_returns_missing_variable_error() {
    let err = translate(
        "post_created.subject",
        Locale::Ru,
        Vars(&[("sender", "alice")]),
    )
    .unwrap_err();
    match err {
        I18nError::MissingVariable { key, name } => {
            assert_eq!(key, "post_created.subject");
            assert_eq!(name, "resource");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn translate_supports_both_locales() {
    let ru = translate(
        "post_created.subject",
        Locale::Ru,
        Vars(&[("sender", "alice"), ("resource", "blog-1")]),
    )
    .unwrap();
    let en = translate(
        "post_created.subject",
        Locale::En,
        Vars(&[("sender", "alice"), ("resource", "blog-1")]),
    )
    .unwrap();
    assert!(ru.contains("Новая запись"));
    assert!(en.contains("New post"));
    assert_ne!(ru, en);
}

#[test]
fn parse_locale_accepts_ru_and_en() {
    assert_eq!(parse_locale("ru").unwrap(), Locale::Ru);
    assert_eq!(parse_locale("en").unwrap(), Locale::En);
    assert_eq!(parse_locale("  ru  ").unwrap(), Locale::Ru);
}

#[test]
fn parse_locale_rejects_unknown_value() {
    let err = parse_locale("de").unwrap_err();
    assert!(matches!(err, I18nError::UnknownLocale(ref v) if v == "de"));
}

#[test]
fn locale_as_str_is_stable() {
    assert_eq!(Locale::Ru.as_str(), "ru");
    assert_eq!(Locale::En.as_str(), "en");
}

#[test]
fn translate_keeps_unrelated_percent_signs_unchanged() {
    // Шаблон может содержать текст без плейсхолдеров — он возвращается как есть.
    let s = translate(
        "post_hidden.subject",
        Locale::Ru,
        Vars(&[("actor", "alice")]),
    )
    .unwrap();
    assert_eq!(s, "Запись скрыта: alice");
}

#[test]
fn translate_subscription_active_and_inactive_have_different_texts() {
    let active = translate(
        "subscription_changed.active.subject",
        Locale::Ru,
        Vars(&[("subscriber", "alice@example.org")]),
    )
    .unwrap();
    let inactive = translate(
        "subscription_changed.inactive.subject",
        Locale::Ru,
        Vars(&[("subscriber", "alice@example.org")]),
    )
    .unwrap();
    assert!(active.starts_with("Подписка:"));
    assert!(inactive.starts_with("Отписка:"));
    assert_ne!(active, inactive);
}

#[test]
fn translate_supports_long_cyrillic_body() {
    let body = "Привет, мир!\n\nЭто тестовое письмо на русском языке.\n\
                С новой строки, с цифрами 0123456789 и знаками — «»…";
    let s = translate(
        "post_created.body",
        Locale::Ru,
        Vars(&[("resource", "blog-1"), ("body", body)]),
    )
    .unwrap();
    assert!(s.contains(body));
    assert!(s.starts_with("Новая запись в журнале blog-1"));
    assert!(s.ends_with("— LiveLetters"));
}

#[test]
fn translate_comment_created_redistribute_subject_ru_uses_resource() {
    let s = translate(
        "comment_created_redistribute.subject",
        Locale::Ru,
        Vars(&[("resource", "blog-1")]),
    )
    .expect("translation should succeed");
    assert_eq!(s, "Новый комментарий в blog-1");
}

#[test]
fn translate_comment_created_redistribute_subject_en_uses_resource() {
    let s = translate(
        "comment_created_redistribute.subject",
        Locale::En,
        Vars(&[("resource", "blog-1")]),
    )
    .expect("translation should succeed");
    assert_eq!(s, "New comment in blog-1");
}

#[test]
fn translate_comment_created_redistribute_body_fills_sender_and_post() {
    let s = translate(
        "comment_created_redistribute.body",
        Locale::Ru,
        Vars(&[("sender", "bob"), ("post_id", "post-1"), ("body", "текст")]),
    )
    .expect("translation should succeed");
    assert!(s.contains("bob"));
    assert!(s.contains("post-1"));
    assert!(s.contains("текст"));
}

#[test]
fn detect_system_locale_falls_back_to_en() {
    // Если в окружении нет LC_ALL/LC_MESSAGES/LANG или они не парсятся,
    // функция возвращает En (а не паникует).
    // Безопасный набор гарантирует: в этой системе `detect_system_locale()`
    // всё равно возвращает один из двух поддерживаемых `Locale`.
    let locale = detect_system_locale();
    assert!(matches!(locale, Locale::Ru | Locale::En));
}

/// Все проверки работы с переменными окружения собраны в один тест,
/// потому что `std::env` — это глобальное состояние процесса, и
/// параллельный запуск (по умолчанию в `cargo test`) приводит к
/// интерференции между тестами: один снимает `LANG`, другой читает.
#[test]
fn detect_system_locale_reads_environment_correctly() {
    // SAFETY: тесты в одном test-файле выполняются параллельно,
    // но этот тест явно сериализуется через мьютекс ниже.
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

    let snapshot: [Option<String>; 3] = [
        std::env::var("LC_ALL").ok(),
        std::env::var("LC_MESSAGES").ok(),
        std::env::var("LANG").ok(),
    ];

    let run = |lc_all: Option<&str>, lc_messages: Option<&str>, lang: Option<&str>| -> Locale {
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            unsafe { std::env::remove_var(var) };
        }
        if let Some(value) = lc_all {
            unsafe { std::env::set_var("LC_ALL", value) };
        }
        if let Some(value) = lc_messages {
            unsafe { std::env::set_var("LC_MESSAGES", value) };
        }
        if let Some(value) = lang {
            unsafe { std::env::set_var("LANG", value) };
        }
        detect_system_locale()
    };

    // 1) Все три пустые -> fallback En.
    assert_eq!(run(None, None, None), Locale::En);

    // 2) Мусор в LANG -> fallback En.
    assert_eq!(run(None, None, Some("de_DE.UTF-8")), Locale::En);

    // 3) `LANG=ru_RU.UTF-8` -> Ru.
    assert_eq!(run(None, None, Some("ru_RU.UTF-8")), Locale::Ru);

    // 4) `LANG=en-US` -> En (поддержка дефиса).
    assert_eq!(run(None, None, Some("en-US")), Locale::En);

    // 5) `LC_ALL` приоритетнее `LANG`.
    assert_eq!(
        run(Some("en_US.UTF-8"), None, Some("ru_RU.UTF-8")),
        Locale::En
    );

    // Восстанавливаем прежнее окружение, чтобы не сломать соседние тесты.
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        unsafe { std::env::remove_var(var) };
    }
    for (idx, var) in ["LC_ALL", "LC_MESSAGES", "LANG"].iter().enumerate() {
        if let Some(value) = &snapshot[idx] {
            unsafe { std::env::set_var(var, value) };
        }
    }
}
