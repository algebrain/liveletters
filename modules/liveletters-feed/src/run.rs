use std::error::Error;

use liveletters_app_core::{GetHomeFeedQuery, get_home_feed};
use liveletters_config::load_identity;
use liveletters_output::CommandContext;
use liveletters_store::Store;

use crate::print::print_feed;
use crate::{Args, FeedError};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), FeedError> {
    let store = Store::open_for_home_dir(&ctx.home)?;
    let feed = get_home_feed(&store, GetHomeFeedQuery)?;
    let identity = load_identity(&ctx.home, &ctx.identity_name).ok();
    let display = identity
        .map(|i| i.display_name().to_owned())
        .unwrap_or_else(|| ctx.identity_name.clone());
    print_feed(&feed, &display, args.limit);
    Ok(())
}
