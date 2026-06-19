use liveletters_app_core::{AppCore, FriendCommand};
use liveletters_store::Store;
use liveletters_utils::time::unix_now;

use crate::{Args, FriendError};

pub fn run(
    ctx: &liveletters_output::CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

fn run_inner(ctx: &liveletters_output::CommandContext, args: &Args) -> Result<(), FriendError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let user = store
        .get_user_settings_record(&ctx.identity_name)?
        .ok_or_else(|| FriendError::IdentityNotFound(ctx.identity_name.clone()))?;
    let core = AppCore::new(&store);
    let result = core.friend(FriendCommand {
        profile_id: &ctx.identity_name,
        owner_resource_address: &user.author_email,
        friend_address: &args.address,
        created_at: unix_now(),
    })?;

    if result.subscription_requested {
        println!("запрошено добавление в друзья: {}", result.friend_address);
    } else {
        println!("добавлен в друзья: {}", result.friend_address);
    }
    Ok(())
}
