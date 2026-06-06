use std::error::Error;

use liveletters_diagnostics::DiagnosticsReader;
use liveletters_output::CommandContext;
use liveletters_store::Store;

use crate::print::{print_doctor, print_doctor_verbose};
use crate::{Args, DoctorError};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args.verbose).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, verbose: bool) -> Result<(), DoctorError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let reader = DiagnosticsReader::new(&store);
    let snap = reader.build_snapshot()?;
    if verbose {
        print_doctor_verbose(&snap, &store, &ctx.home)?;
    } else {
        print_doctor(&snap);
    }
    Ok(())
}
