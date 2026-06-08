mod common;

use std::error::Error;

use liveletters_init::{Args as InitArgs, run};

#[test]
fn init_creates_expected_layout() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (ctx, _tmp) = common::init_ctx();
    run(&ctx, &InitArgs { force: false })?;

    let home = &ctx.home;
    assert!(
        home.join("liveletters.sqlite3").exists(),
        "liveletters.sqlite3 не создан"
    );
    assert!(
        home.join("mail-password-obfuscation.key").exists(),
        "mail-password-obfuscation.key не создан"
    );
    assert!(
        !home.join("identities").join("default.toml").exists(),
        "init не должен создавать пользователя default"
    );
    assert!(
        !home.join("current-user").exists(),
        "init не должен выбирать текущего пользователя"
    );
    for sub in ["drafts", "inbox", "outbox-staged", "logs"] {
        assert!(home.join(sub).is_dir(), "каталог {sub} не создан");
    }
    Ok(())
}

#[test]
fn init_fails_when_home_not_empty_without_force() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (ctx, _tmp) = common::init_ctx();
    std::fs::write(ctx.home.join("junk"), b"x")?;

    let err = run(&ctx, &InitArgs { force: false }).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("уже существует"),
        "неожиданное сообщение: {msg}"
    );
    Ok(())
}

#[test]
fn init_force_overwrites_existing_files() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (ctx, _tmp) = common::init_ctx();
    std::fs::write(ctx.home.join("junk"), b"x")?;

    run(&ctx, &InitArgs { force: true })?;
    assert!(ctx.home.join("liveletters.sqlite3").exists());
    Ok(())
}

#[test]
fn init_is_idempotent_when_home_empty() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (ctx, _tmp) = common::init_ctx();
    run(&ctx, &InitArgs { force: false })?;
    let second = run(&ctx, &InitArgs { force: false });
    // Повторный init поверх только что инициализированного каталога без --force — ошибка.
    assert!(second.is_err());
    Ok(())
}
