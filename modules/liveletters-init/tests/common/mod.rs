use liveletters_init::CommandContext;
use tempfile::TempDir;

pub fn init_ctx() -> (CommandContext, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let ctx = CommandContext {
        home: tmp.path().to_path_buf(),
        identity_name: "default".to_owned(),
    };
    (ctx, tmp)
}
