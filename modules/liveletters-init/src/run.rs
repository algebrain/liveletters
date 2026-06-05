use std::{error::Error, fs, path::Path};

use liveletters_secret_box::{SecretBox, default_key_path};
use liveletters_store::Store;

use crate::{Args, CommandContext, InitError};

/// Каталоги, которые `init` создаёт внутри `home`.
const SUBDIRS: &[&str] = &["identities", "drafts", "inbox", "outbox-staged", "logs"];

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_inner(ctx.home.as_path(), args.force)
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

fn run_inner(home: &Path, force: bool) -> Result<(), InitError> {
    ensure_home_empty(home, force)?;

    fs::create_dir_all(home)?;
    for sub in SUBDIRS {
        fs::create_dir_all(home.join(sub))?;
    }

    let _store = Store::open_for_home_dir(home)?;
    drop(_store);

    let key_path = default_key_path(home);
    let _box = SecretBox::open_or_create(&key_path)?;
    drop(_box);

    println!("инициализирован {}", home.display());
    println!("создан lltt.db");
    println!("создан mail-password-obfuscation.key");
    println!("создан каталог identities");
    println!("создан каталог drafts");

    Ok(())
}

fn ensure_home_empty(home: &Path, force: bool) -> Result<(), InitError> {
    if !home.exists() {
        return Ok(());
    }
    let is_empty = home.read_dir()?.next().is_none();
    if is_empty || force {
        return Ok(());
    }
    Err(InitError::AlreadyExists(home.to_path_buf()))
}
