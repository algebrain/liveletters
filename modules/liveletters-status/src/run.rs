use std::error::Error;

use liveletters_output::CommandContext;
use liveletters_store::Store;

use crate::print::{StatusCounts, print_status};
use crate::{Args, StatusError};

pub fn run(ctx: &CommandContext, _args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext) -> Result<(), StatusError> {
    let store = Store::open_for_home_dir(&ctx.home)?;
    let counts = StatusCounts {
        posts: store.count_posts()?,
        comments: store.count_comments()?,
        deferred: store.count_deferred_events()?,
        outbox: store.count_outbox()?,
        last_activity: store.newest_post_created_at()?,
    };
    print_status(&counts);
    Ok(())
}
