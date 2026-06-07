mod error;
mod locale;
mod templates;
mod translate;

pub use error::I18nError;
pub use locale::{Locale, detect_system_locale, parse_locale};
pub use translate::{Vars, translate};

pub fn crate_name() -> &'static str {
    "liveletters-i18n"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-i18n");
    }
}
