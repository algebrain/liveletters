use std::collections::HashSet;
use std::error::Error;

use liveletters_output::CommandContext;
use liveletters_store::{PostRecord, Store};

use crate::print::print_feed;
use crate::{Args, FeedError};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), FeedError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let subscribed: HashSet<String> = store
        .list_local_subscriptions(&ctx.identity_name)?
        .into_iter()
        .collect();
    let owned: HashSet<String> = store
        .list_resources_owned(&ctx.identity_name)?
        .into_iter()
        .collect();
    let display_name = store
        .get_user_settings_record(&ctx.identity_name)?
        .map(|r| r.nickname)
        .unwrap_or_default();

    let posts = store
        .list_posts()?
        .into_iter()
        .filter(|post| is_subscription_post(post, &subscribed, &owned))
        .collect::<Vec<_>>();

    print_feed(&posts, &display_name, args.limit);
    Ok(())
}

fn is_subscription_post(
    post: &PostRecord,
    subscribed: &HashSet<String>,
    owned: &HashSet<String>,
) -> bool {
    subscribed.contains(&post.resource_id) && !owned.contains(&post.resource_id)
}
