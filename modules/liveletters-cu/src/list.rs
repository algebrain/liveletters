use crate::error::CuError;

pub fn run(ctx: &liveletters_output::CommandContext) -> Result<(), CuError> {
    let users_dir = ctx.home.join("users");
    if !users_dir.exists() {
        return Ok(());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&users_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && entry.path().join("liveletters.sqlite3").exists()
            && let Some(name) = entry.file_name().to_str()
        {
            {
                names.push(name.to_owned());
            }
        }
    }
    names.sort();
    for name in &names {
        println!("{name}");
    }
    Ok(())
}
