use std::error::Error;

use liveletters_app_core::{GetCurrentUserPostsQuery, get_current_user_posts};
use liveletters_config::load_identity;
use liveletters_output::CommandContext;
use liveletters_store::Store;

use crate::print::print_posts;
use crate::{Args, PostsError};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), PostsError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let identity = load_identity(&ctx.home, &ctx.identity_name)?;
    let posts = get_current_user_posts(
        &store,
        GetCurrentUserPostsQuery {
            author_id: identity.account_id(),
        },
    )?;
    print_posts(&posts, identity.display_name(), args.limit);
    Ok(())
}
