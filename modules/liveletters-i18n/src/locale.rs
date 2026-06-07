use crate::I18nError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    Ru,
    En,
}

impl Locale {
    pub fn as_str(self) -> &'static str {
        match self {
            Locale::Ru => "ru",
            Locale::En => "en",
        }
    }
}

pub fn parse_locale(value: &str) -> Result<Locale, I18nError> {
    match value.trim() {
        "ru" => Ok(Locale::Ru),
        "en" => Ok(Locale::En),
        other => Err(I18nError::UnknownLocale(other.to_owned())),
    }
}

/// Определяет локаль по переменным окружения `LC_ALL`, `LC_MESSAGES`, `LANG`
/// (в этом порядке). Если ничего не задано или значение не входит в
/// поддерживаемые локали — возвращает `Locale::En`.
pub fn detect_system_locale() -> Locale {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(value) = std::env::var_os(var)
            && let Some(locale) = parse_env_locale(&value.to_string_lossy())
        {
            return locale;
        }
    }
    Locale::En
}

fn parse_env_locale(value: &str) -> Option<Locale> {
    let lang = value.split(['_', '-', '.']).next()?.trim();
    parse_locale(lang).ok()
}
