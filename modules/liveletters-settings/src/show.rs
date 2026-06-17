use std::path::Path;

use liveletters_config::load_global;
use liveletters_store::Store;

use crate::error::SettingsError;
use crate::print::print_log_config;
use crate::print::print_settings;

pub fn run(home: &Path, state_home: &Path, identity_name: &str) -> Result<(), SettingsError> {
    let store = Store::open_for_home_dir(state_home)?;
    let user = store.get_user_settings_record(identity_name)?;
    // Ник берём из authors (FK user_settings.author_email → authors.email).
    let author = user
        .as_ref()
        .and_then(|u| store.get_author(&u.author_email).ok().flatten());
    let mail = store.get_mail_settings_record(identity_name)?;
    print_settings(user.as_ref(), author.as_ref(), mail.as_ref());
    let global = load_global(home)?;
    print_log_config(&global.log);
    Ok(())
}
