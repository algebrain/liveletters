use std::path::Path;

use liveletters_store::Store;

use crate::error::SettingsError;
use crate::print::print_settings;

pub fn run(home: &Path, identity_name: &str) -> Result<(), SettingsError> {
    let store = Store::open_for_home_dir(home)?;
    let user = store.get_user_settings_record(identity_name)?;
    let mail = store.get_mail_settings_record(identity_name)?;
    print_settings(user.as_ref(), mail.as_ref());
    Ok(())
}
