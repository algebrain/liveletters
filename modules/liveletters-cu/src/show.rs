use crate::error::CuError;
use liveletters_output::print_identity_from_db;

pub fn run(
    ctx: &liveletters_output::CommandContext,
    name: &str,
    reveal: bool,
) -> Result<(), CuError> {
    let store = liveletters_store::Store::open_for_home_dir(ctx.home.join("users").join(name))?;
    let user = store
        .get_user_settings_record(name)?
        .ok_or_else(|| CuError::UnknownIdentity(name.to_owned()))?;
    // Ник берём из `authors` (FK user_settings.author_email → authors.email).
    let author = store.get_author(&user.author_email)?;
    let mail = store.get_mail_settings_record(name)?;
    let receive = store.list_receive_addresses(name)?;
    let resources = store.list_resources_owned(name)?;
    let local_subs = store.list_local_subscriptions(name)?;

    print_identity_from_db(
        &user,
        author.as_ref(),
        mail.as_ref(),
        &receive,
        &resources,
        &local_subs,
        reveal,
    );
    Ok(())
}
