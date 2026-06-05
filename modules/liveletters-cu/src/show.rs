use crate::error::CuError;

pub fn run(
    ctx: &liveletters_output::CommandContext,
    name: &str,
    reveal: bool,
) -> Result<(), CuError> {
    let cfg = liveletters_config::load_identity(&ctx.home, name)?;
    liveletters_output::print_identity(&cfg, reveal);
    Ok(())
}
