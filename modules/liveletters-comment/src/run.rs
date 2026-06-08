use std::error::Error;
use std::io::{self};

use liveletters_app_core::{AppCore, CreateCommentFromIdentityCommand, Identity};
use liveletters_output::{CommandContext, parse_visibility, read_body};
use liveletters_store::Store;

use crate::args::{CommentAction, NewArgs};
use crate::error::CommentError;
use crate::{Args, print_created};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), CommentError> {
    match &args.action {
        CommentAction::New(new_args) => run_new(ctx, new_args),
    }
}

fn run_new(ctx: &CommandContext, args: &NewArgs) -> Result<(), CommentError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let user = store
        .get_user_settings_record(&ctx.identity_name)?
        .ok_or_else(|| CommentError::IdentityNotFound(ctx.identity_name.clone()))?;
    let identity = Identity {
        account_id: format!("acct_{}", &ctx.identity_name),
        publish: user.email_address,
    };

    let body = read_body(args.body_file.as_deref(), &mut io::stdin().lock())
        .map_err(CommentError::IoFromOutput)?;
    if body.trim().is_empty() {
        return Err(CommentError::EmptyBody);
    }

    let visibility = parse_visibility(&args.visibility).map_err(CommentError::UnknownVisibility)?;

    let core = AppCore::new(&store);
    let result = core.create_comment_from_identity(CreateCommentFromIdentityCommand {
        profile_id: &ctx.identity_name,
        identity: &identity,
        post_id: &args.post,
        parent_comment_id: args.parent.as_deref(),
        body: &body,
        visibility,
    })?;

    print_created(result.comment().id().as_str());
    Ok(())
}
