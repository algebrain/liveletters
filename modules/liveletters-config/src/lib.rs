mod error;
mod global;
mod identity;
mod io;
mod mapping;

pub use error::ConfigError;
pub use global::GlobalConfig;
pub use identity::{
    IdentityConfig, IdentityMeta, ImapSettings, MailSecurity, MailSettings, SmtpSettings,
};
pub use io::{
    current_user_path, list_identities, load_global, load_identity, read_current_identity,
    save_identity, write_current_identity,
};
pub use mapping::{map_identity_to_settings, settings_to_identity};

pub fn crate_name() -> &'static str {
    "liveletters-config"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn crate_name_is_set() {
        assert_eq!(crate_name(), "liveletters-config");
    }
}
