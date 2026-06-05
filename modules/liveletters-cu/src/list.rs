use crate::error::CuError;

pub fn run(ctx: &liveletters_output::CommandContext) -> Result<(), CuError> {
    let names = liveletters_config::list_identities(&ctx.home)?;
    for name in names {
        println!("{name}");
    }
    Ok(())
}
