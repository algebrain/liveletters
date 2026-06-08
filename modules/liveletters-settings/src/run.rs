use std::error::Error;

use liveletters_output::CommandContext;

use crate::args::SettingsAction;
use crate::error::SettingsError;
use crate::{Args, set, show};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx, args).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(ctx: &CommandContext, args: &Args) -> Result<(), SettingsError> {
    let action = parse_action(&args.tokens)?;
    match action {
        SettingsAction::Show => show::run(&ctx.home, &ctx.state_home, &ctx.identity_name),
        SettingsAction::Set { key, value } => {
            if let Some(field) = key.strip_prefix("log.") {
                set::run_log_field(&ctx.home, field, &value)
            } else {
                set::run_db_field(&ctx.home, &ctx.state_home, &ctx.identity_name, &key, &value)
            }
        }
    }
}

fn parse_action(tokens: &[String]) -> Result<SettingsAction, SettingsError> {
    match tokens.first().map(String::as_str) {
        None | Some("show") => Ok(SettingsAction::Show),
        Some("set") => {
            if tokens.len() == 3 {
                Ok(SettingsAction::Set {
                    key: tokens[1].clone(),
                    value: tokens[2].clone(),
                })
            } else {
                Err(SettingsError::InvalidArgs(
                    "`set` требует два аргумента: <ключ> <значение>".into(),
                ))
            }
        }
        Some(other) => Err(SettingsError::InvalidArgs(format!(
            "неизвестная подкоманда: {other}"
        ))),
    }
}
