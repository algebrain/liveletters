use crate::{CuError, name::validate_user_name};

pub fn run(
    ctx: &liveletters_output::CommandContext,
    name: &str,
    force: bool,
) -> Result<(), CuError> {
    validate_user_name(name)?;

    let drafts = ctx.home.join("drafts");
    std::fs::create_dir_all(&drafts)?;
    let path = drafts.join(format!("{name}.toml"));
    if path.exists() && !force {
        return Err(CuError::InvalidArgs(format!(
            "черновик {} уже существует; используйте --force",
            path.display()
        )));
    }

    let raw = draft_toml(name);
    std::fs::write(&path, &raw)?;

    println!("создан черновик {}", path.display());
    println!();
    print!("{raw}");
    Ok(())
}

fn draft_toml(name: &str) -> String {
    let display = capitalize_ascii(name);
    format!(
        r#"account_id = "acct_{name}"
display_name = "{display}"

[mail]
publish = "{name}@example.org"
receive = ["{name}@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "{name}@example.org"
password = ""
pwd_obfuscate = true
hello_domain = "example.org"

[mail.imap]
host = "imap.example.org"
port = 993
security = "tls"
username = "{name}@example.org"
password = ""
pwd_obfuscate = true
mailbox = "INBOX"

[meta]
resources_owned = ["{name}@example.org"]
subscriptions = []
"#
    )
}

fn capitalize_ascii(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + chars.as_str()
}
