use crate::error::CuError;
use crate::run::read_current_user;

pub fn run(ctx: &liveletters_output::CommandContext) -> Result<(), CuError> {
    let name = read_current_user(ctx.home.as_path())?;
    println!("{name}");
    Ok(())
}
