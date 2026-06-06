use std::error::Error;

use liveletters_config::{read_current_identity, write_current_identity};

use crate::{Args, CommandContext, CuAction, CuError};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_current(ctx, args)
}

pub fn run_current(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    let action = parse_current_action(&args.tokens)
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    match action {
        CuAction::Current => super::current::run(ctx)?,
        CuAction::Switch { name } => super::switch::run(ctx, &name)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?,
        CuAction::ShowCurrent { reveal } => {
            let name = read_current_user(&ctx.home)
                .map_err(|e| Box::new(CuError::Config(e)) as Box<dyn Error + Send + Sync>)?;
            super::show::run(ctx, &name, reveal)
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        }
        CuAction::Posts { limit } => {
            liveletters_posts::run(ctx, &liveletters_posts::Args { limit })?
        }
        _ => unreachable!("parse_current_action returned user-only action"),
    }
    Ok(())
}

pub fn run_user(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    let action =
        parse_user_action(&args.tokens).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    match action {
        CuAction::List => {
            super::list::run(ctx).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?
        }
        CuAction::Init { name, force } => super::user_init::run(ctx, &name, force)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?,
        CuAction::Show { name, reveal } => super::show::run(ctx, &name, reveal)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?,
        CuAction::Add { name, from } => {
            let from = from.unwrap_or_else(|| default_draft_path(ctx, &name));
            super::add::run(ctx, &name, &from)
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?
        }
        CuAction::Rm { name, yes } => super::rm::run(ctx, &name, yes)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?,
        _ => unreachable!("parse_user_action returned current-user action"),
    }
    Ok(())
}

fn parse_current_action(tokens: &[String]) -> Result<CuAction, CuError> {
    match tokens.first().map(String::as_str) {
        None => Ok(CuAction::Current),
        Some("show") if tokens.len() == 1 => Ok(CuAction::ShowCurrent { reveal: false }),
        Some("show") if tokens.len() == 2 && tokens[1] == "--reveal" => {
            Ok(CuAction::ShowCurrent { reveal: true })
        }
        Some("posts") => parse_posts(&tokens[1..]),
        Some("show") => Err(CuError::UseUserCommand("lltt user show <имя>".to_owned())),
        Some("list") => Err(CuError::UseUserCommand("lltt user list".to_owned())),
        Some("add") => Err(CuError::UseUserCommand("lltt user add <имя>".to_owned())),
        Some("rm") => Err(CuError::UseUserCommand(
            "lltt user rm <имя> --yes".to_owned(),
        )),
        Some(other) => {
            if tokens.len() != 1 {
                return Err(CuError::ConflictingArgs);
            }
            Ok(CuAction::Switch {
                name: other.to_owned(),
            })
        }
    }
}

fn parse_posts(rest: &[String]) -> Result<CuAction, CuError> {
    let mut limit: Option<usize> = None;
    let mut iter = rest.iter();
    while let Some(token) = iter.next() {
        if let Some(value) = token.strip_prefix("--limit=") {
            limit = Some(parse_limit(value)?);
        } else if token == "--limit" {
            let value = iter
                .next()
                .ok_or_else(|| CuError::InvalidArgs("`--limit` требует значение".to_owned()))?;
            limit = Some(parse_limit(value)?);
        } else if let Some(value) = token.strip_prefix("--") {
            return Err(CuError::InvalidArgs(format!("неизвестный флаг: --{value}")));
        } else {
            return Err(CuError::InvalidArgs(format!(
                "`posts` не принимает позиционный аргумент: {token}"
            )));
        }
    }
    Ok(CuAction::Posts { limit })
}

fn parse_limit(value: &str) -> Result<usize, CuError> {
    value.parse::<usize>().map_err(|_| {
        CuError::InvalidArgs(format!("`--limit` требует целое число, получили: {value}"))
    })
}

fn parse_user_action(tokens: &[String]) -> Result<CuAction, CuError> {
    match tokens.first().map(String::as_str) {
        None => Err(CuError::InvalidArgs(
            "`user` требует подкоманду: list, init, show, add или rm".to_owned(),
        )),
        Some("list") => {
            if tokens.len() != 1 {
                return Err(CuError::InvalidArgs(format!(
                    "`list` не принимает аргументов, получили: {tokens:?}"
                )));
            }
            Ok(CuAction::List)
        }
        Some("init") => parse_init(&tokens[1..]),
        Some("show") => parse_show(&tokens[1..]),
        Some("add") => parse_add(&tokens[1..]),
        Some("rm") => parse_rm(&tokens[1..]),
        Some(other) => Err(CuError::InvalidArgs(format!(
            "неизвестная подкоманда `user {other}`"
        ))),
    }
}

fn parse_init(rest: &[String]) -> Result<CuAction, CuError> {
    let mut name: Option<String> = None;
    let mut force = false;
    for token in rest {
        if token == "--force" {
            force = true;
        } else if let Some(value) = token.strip_prefix("--") {
            return Err(CuError::InvalidArgs(format!("неизвестный флаг: --{value}")));
        } else if name.is_some() {
            return Err(CuError::InvalidArgs(format!(
                "лишний позиционный аргумент: {token}"
            )));
        } else {
            name = Some(token.clone());
        }
    }
    let name =
        name.ok_or_else(|| CuError::InvalidArgs("`init` требует имя пользователя".to_owned()))?;
    Ok(CuAction::Init { name, force })
}

fn parse_show(rest: &[String]) -> Result<CuAction, CuError> {
    let mut name: Option<String> = None;
    let mut reveal = false;
    for token in rest {
        if let Some(value) = token.strip_prefix("--reveal") {
            if !value.is_empty() {
                return Err(CuError::InvalidArgs(format!(
                    "неожиданный аргумент: {token}"
                )));
            }
            reveal = true;
        } else if let Some(value) = token.strip_prefix("--") {
            return Err(CuError::InvalidArgs(format!("неизвестный флаг: --{value}")));
        } else if name.is_some() {
            return Err(CuError::InvalidArgs(format!(
                "лишний позиционный аргумент: {token}"
            )));
        } else {
            name = Some(token.clone());
        }
    }
    let name =
        name.ok_or_else(|| CuError::InvalidArgs("`show` требует имя идентичности".to_owned()))?;
    Ok(CuAction::Show { name, reveal })
}

fn parse_add(rest: &[String]) -> Result<CuAction, CuError> {
    let mut name: Option<String> = None;
    let mut from: Option<std::path::PathBuf> = None;
    let mut iter = rest.iter();
    while let Some(token) = iter.next() {
        if let Some(value) = token.strip_prefix("--from=") {
            from = Some(std::path::PathBuf::from(value));
        } else if token == "--from" {
            let next = iter
                .next()
                .ok_or_else(|| CuError::InvalidArgs("`--from` требует значение".to_owned()))?;
            from = Some(std::path::PathBuf::from(next));
        } else if let Some(value) = token.strip_prefix("--") {
            return Err(CuError::InvalidArgs(format!("неизвестный флаг: --{value}")));
        } else if name.is_some() {
            return Err(CuError::InvalidArgs(format!(
                "лишний позиционный аргумент: {token}"
            )));
        } else {
            name = Some(token.clone());
        }
    }
    let name =
        name.ok_or_else(|| CuError::InvalidArgs("`add` требует имя идентичности".to_owned()))?;
    Ok(CuAction::Add { name, from })
}

fn default_draft_path(ctx: &CommandContext, name: &str) -> std::path::PathBuf {
    ctx.home.join("drafts").join(format!("{name}.toml"))
}

fn parse_rm(rest: &[String]) -> Result<CuAction, CuError> {
    let mut name: Option<String> = None;
    let mut yes = false;
    for token in rest {
        if token == "--yes" {
            yes = true;
        } else if let Some(value) = token.strip_prefix("--") {
            return Err(CuError::InvalidArgs(format!("неизвестный флаг: --{value}")));
        } else if name.is_some() {
            return Err(CuError::InvalidArgs(format!(
                "лишний позиционный аргумент: {token}"
            )));
        } else {
            name = Some(token.clone());
        }
    }
    let name =
        name.ok_or_else(|| CuError::InvalidArgs("`rm` требует имя идентичности".to_owned()))?;
    Ok(CuAction::Rm { name, yes })
}

pub fn ensure_name_exists(home: &std::path::Path, name: &str) -> Result<(), crate::error::CuError> {
    let path = home.join("identities").join(format!("{name}.toml"));
    if !path.exists() {
        return Err(crate::error::CuError::Config(
            liveletters_config::ConfigError::UnknownIdentity(name.to_owned()),
        ));
    }
    Ok(())
}

pub fn read_current_user(
    home: &std::path::Path,
) -> Result<String, liveletters_config::ConfigError> {
    read_current_identity(home)
}

pub fn write_current_user(
    home: &std::path::Path,
    name: &str,
) -> Result<(), liveletters_config::ConfigError> {
    write_current_identity(home, name)
}
