use crate::error::CuError;
use crate::run::{ensure_name_exists, write_current_user};

pub fn run(ctx: &liveletters_output::CommandContext, name: &str) -> Result<(), CuError> {
    ensure_name_exists(&ctx.home, name)?;
    write_current_user(&ctx.home, name)?;
    println!("текущий пользователь: {name}");
    Ok(())
}
