use std::error::Error;
use std::io::{self};

use liveletters_app_core::{AppCore, CreatePostFromIdentityCommand, Identity};
use liveletters_config::{MailSettings, load_identity};
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
    let store = Store::open_for_home_dir(&ctx.home)?;
    let identity_cfg = load_identity(&ctx.home, &ctx.identity_name)?;
    let identity = identity_from_config(&identity_cfg.mail, &identity_cfg.account_id);

    let body = read_body(args.body_file.as_deref(), &mut io::stdin().lock())
        .map_err(PostError::IoFromOutput)?;
    if body.trim().is_empty() {
        return Err(PostError::EmptyBody);
    }

    let visibility = parse_visibility(&args.visibility).map_err(PostError::UnknownVisibility)?;

    let core = AppCore::new(&store);
    let result = core.create_post_from_identity(CreatePostFromIdentityCommand {
        identity: &identity,
        body: &body,
        visibility,
    })?;

    print_created(result.post().id().as_str());
    Ok(())
}

fn identity_from_config(mail: &MailSettings, account_id: &str) -> Identity {
    Identity {
        account_id: account_id.to_owned(),
        publish: mail.publish().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_from_config_uses_publish_and_account_id() {
        let mail = MailSettings {
            publish: "x-publish@example.org".to_owned(),
            receive: vec!["x-feed@example.org".to_owned()],
            smtp: None,
            imap: None,
        };
        let id = identity_from_config(&mail, "x");
        assert_eq!(id.account_id, "x");
        assert_eq!(id.publish, "x-publish@example.org");
    }
}
