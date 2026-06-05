use std::path::PathBuf;

/// Контекст выполнения команды `lltt`.
///
/// Собирается в [`apps/lltt/src/main.rs`] один раз на старте процесса и
/// передаётся во все командные крейты через `pub fn run(&CommandContext, &Args)`.
///
/// [`apps/lltt/src/main.rs`]: ../../../apps/lltt/src/main.rs
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Разрешённый `LIVELETTERS_HOME` (или `~/.liveletters/` по умолчанию).
    pub home: PathBuf,
    /// Результат [`liveletters_config::resolve_identity_name`].
    ///
    /// [`liveletters_config::resolve_identity_name`]: ../../liveletters-config/src/lib.rs
    pub identity_name: String,
}
