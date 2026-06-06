use std::collections::HashSet;
use std::error::Error;

use liveletters_config::load_identity;
use liveletters_output::CommandContext;
use liveletters_store::{PostRecord, Store};

use crate::print::print_feed;
use crate::{Args, FeedError};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), FeedError> {
    let identity = load_identity(&ctx.home, &ctx.identity_name)?;
    let subscribed = identity
        .subscriptions()
        .iter()
        .map(|resource| resource.as_str())
        .collect::<HashSet<_>>();
    let owned = identity
        .resources_owned()
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();

    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let posts = store
        .list_posts()?
        .into_iter()
        .filter(|post| is_subscription_post(post, &subscribed, &owned))
        .collect::<Vec<_>>();

    print_feed(&posts, identity.display_name(), args.limit);
    Ok(())
}

fn is_subscription_post(
    post: &PostRecord,
    subscribed: &HashSet<&str>,
    owned: &HashSet<&str>,
) -> bool {
    subscribed.contains(post.resource_id.as_str()) && !owned.contains(post.resource_id.as_str())
}
