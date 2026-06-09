use std::error::Error;

use liveletters_app_core::{GetCurrentUserPostsQuery, get_current_user_posts};
use liveletters_output::CommandContext;
use liveletters_store::Store;

use crate::print::print_posts;
use crate::{Args, PostsError};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), PostsError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let user = store
        .get_user_settings_record(&ctx.identity_name)?
        .ok_or_else(|| PostsError::IdentityNotFound(ctx.identity_name.clone()))?;
    let account_id = format!("acct_{}", &ctx.identity_name);
    let posts = get_current_user_posts(
        &store,
        GetCurrentUserPostsQuery {
            author_id: &account_id,
        },
    )?;
    print_posts(&posts, &user.nickname, args.limit);
    Ok(())
}
