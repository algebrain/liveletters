use crate::error::CuError;
use crate::run::read_current_user;

pub fn run(ctx: &liveletters_output::CommandContext, name: &str, yes: bool) -> Result<(), CuError> {
    if !yes {
        return Err(CuError::InvalidArgs(
            "нужен флаг --yes для подтверждения удаления".to_owned(),
        ));
    }
    let current = read_current_user(&ctx.home).ok();
    if current.as_deref() == Some(name) {
        return Err(CuError::CannotRemoveCurrent(name.to_owned()));
    }
    let store = liveletters_store::Store::open_for_home_dir(ctx.home.join("users").join(name))?;
    if store.get_user_settings_record(name)?.is_none() {
        return Err(CuError::UnknownIdentity(name.to_owned()));
    }
    drop(store);
    let db_path = ctx
        .home
        .join("users")
        .join(name)
        .join("liveletters.sqlite3");
    std::fs::remove_file(&db_path)?;
    println!("удалён {name}");
    Ok(())
}
