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
    let path = ctx.home.join("identities").join(format!("{name}.toml"));
    if !path.exists() {
        return Err(CuError::Config(
            liveletters_config::ConfigError::UnknownIdentity(name.to_owned()),
        ));
    }
    std::fs::remove_file(&path)?;
    println!("удалён identities/{name}.toml");
    Ok(())
}
