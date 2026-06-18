use std::collections::HashSet;
use std::error::Error;

use liveletters_output::CommandContext;
use liveletters_store::{PostRecord, Store};

use crate::print::{FeedPost, print_feed};
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
    // Ник берём из authors (user_settings.author_email → authors.email).
    let display_name = store
        .get_user_settings_record(&ctx.identity_name)?
        .and_then(|r| store.get_author(&r.author_email).ok().flatten())
        .map(|a| a.nickname)
        .unwrap_or_default();

    let posts = store
        .list_posts()?
        .into_iter()
        .filter(|post| is_subscription_post(post, &subscribed, &owned))
        .map(|post| {
            let author_display = store.format_author_identity(&post.author_email)?;
            Ok(FeedPost::new(post, author_display))
        })
        .collect::<Result<Vec<_>, liveletters_store::StoreError>>()?;

    print_feed(&posts, &display_name, args.limit);
    Ok(())
}

fn is_subscription_post(
    post: &PostRecord,
    subscribed: &HashSet<String>,
    owned: &HashSet<String>,
) -> bool {
    subscribed.contains(&post.resource_email) && !owned.contains(&post.resource_email)
}
