use std::error::Error;

use liveletters_app_core::{GetPendingOutboxQuery, OutboxEntry, PendingOutbox, get_pending_outbox};
use liveletters_output::{CommandContext, print_kv, print_table};
use liveletters_store::Store;

use crate::Args;
use crate::args::OutboxAction;
use crate::error::OutboxError;

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), OutboxError> {
    match &args.action {
        OutboxAction::List => run_list(ctx),
    }
}

fn run_list(ctx: &CommandContext) -> Result<(), OutboxError> {
    let store = Store::open_for_home_dir(&ctx.home)?;
    let pending = get_pending_outbox(&store, GetPendingOutboxQuery)?;

    print_summary(&pending);
    Ok(())
}

pub fn print_summary(pending: &PendingOutbox) {
    let total = pending.entries().len();
    print_kv(&[("неотправленные события", &total.to_string())]);

    if total == 0 {
        println!();
        println!("(пусто)");
        return;
    }

    println!();
    let headers = &["event_id", "event_type", "resource_id"];
    let rows: Vec<Vec<String>> = pending.entries().iter().map(row_from_entry).collect();
    print_table(headers, &rows);
}

fn row_from_entry(entry: &OutboxEntry) -> Vec<String> {
    vec![
        entry.event_id.clone(),
        entry.event_type.clone(),
        entry.resource_id.clone(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_summary_handles_empty_pending() {
        let pending = PendingOutbox::new(vec![]);
        print_summary(&pending);
    }
}
