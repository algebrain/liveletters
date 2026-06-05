use std::error::Error;

use liveletters_output::CommandContext;

use crate::args::InboxAction;
use crate::import;
use crate::list;
use crate::show;
use crate::{Args, InboxError};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), InboxError> {
    match &args.action {
        InboxAction::Import(import_args) => {
            import::run(&ctx.home, &import_args.files)?;
        }
        InboxAction::List(list_args) => {
            list::run(&ctx.home, list_args)?;
        }
        InboxAction::Show(show_args) => {
            show::run(&ctx.home, show_args)?;
        }
    }
    Ok(())
}
