use std::error::Error;
use std::io::{self};

use liveletters_app_core::{AppCore, CreatePostFromIdentityCommand, Identity};
use liveletters_output::{CommandContext, parse_visibility, read_body};
use liveletters_store::Store;

use crate::args::{NewArgs, PostAction};
use crate::error::PostError;
use crate::{Args, print_created};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), PostError> {
    match &args.action {
        PostAction::New(new_args) => run_new(ctx, new_args),
    }
}

fn run_new(ctx: &CommandContext, args: &NewArgs) -> Result<(), PostError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let user = store
        .get_user_settings_record(&ctx.identity_name)?
        .ok_or_else(|| PostError::IdentityNotFound(ctx.identity_name.clone()))?;
    let identity = Identity {
        publish: user.author_email,
    };

    let body = read_body(args.body_file.as_deref(), &mut io::stdin().lock())
        .map_err(PostError::IoFromOutput)?;
    if body.trim().is_empty() {
        return Err(PostError::EmptyBody);
    }

    let visibility = parse_visibility(&args.visibility).map_err(PostError::UnknownVisibility)?;

    let core = AppCore::new(&store);
    let result = core.create_post_from_identity(CreatePostFromIdentityCommand {
        profile_id: &ctx.identity_name,
        identity: &identity,
        body: &body,
        visibility,
    })?;

    print_created(result.post().id().as_str());
    Ok(())
}
