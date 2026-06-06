//! Команда `lltt doctor` — диагностика состояния домашнего каталога.

mod args;
mod error;
pub mod print;
mod run;

pub use args::Args;
pub use error::DoctorError;
pub use liveletters_output::CommandContext;
pub use print::{print_doctor, print_doctor_verbose};
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "doctor";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "диагностика состояния домашнего каталога"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-doctor"
}
