use std::error::Error;
use std::io::{self};

use liveletters_app_core::{AppCore, CreateCommentFromIdentityCommand, Identity, Visibility};
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
    let body = read_body(args.body_file.as_deref(), &mut io::stdin().lock())
        .map_err(CommentError::IoFromOutput)?;
    let visibility = parse_visibility(&args.visibility).map_err(CommentError::UnknownVisibility)?;
    let result = create(ctx, &args.target, &body, visibility)?;
    print_created(result.comment().id().as_str());
    Ok(())
}

/// Создать комментарий по target (id поста или родительского комментария).
/// Дискриминация — по префиксу id. Используется и `lltt answer`,
/// и `lltt comment new` (как синоним).
pub fn create(
    ctx: &CommandContext,
    target: &str,
    body: &str,
    visibility: Visibility,
) -> Result<liveletters_app_core::CreateCommentResult, CommentError> {
    if body.trim().is_empty() {
        return Err(CommentError::EmptyBody);
    }

    let store = Store::open_for_home_dir(&ctx.state_home)?;
    let user = store
        .get_user_settings_record(&ctx.identity_name)?
        .ok_or_else(|| CommentError::IdentityNotFound(ctx.identity_name.clone()))?;
    let identity = Identity {
        publish: user.author_email,
    };

    let core = AppCore::new(&store);

    if target.starts_with("post-") {
        if store.get_post_record(target)?.is_none() {
            return Err(CommentError::PostNotFound(target.to_owned()));
        }
        return core
            .create_comment_from_identity(CreateCommentFromIdentityCommand {
                profile_id: &ctx.identity_name,
                identity: &identity,
                post_id: target,
                parent_comment_id: None,
                body,
                visibility,
            })
            .map_err(CommentError::AppCore);
    }

    if target.starts_with("comment-") {
        let Some(parent) = store.get_comment_record(target)? else {
            return Err(CommentError::CommentNotFound(target.to_owned()));
        };
        return core
            .create_comment_from_identity(CreateCommentFromIdentityCommand {
                profile_id: &ctx.identity_name,
                identity: &identity,
                post_id: &parent.post_id,
                parent_comment_id: Some(target),
                body,
                visibility,
            })
            .map_err(CommentError::AppCore);
    }

    Err(CommentError::InvalidTarget(target.to_owned()))
}
