//! Сетевая синхронизация: `lltt sync`, `lltt sync pull` и `lltt sync push`.
//!
//! Реальная реализация подключается под признаком `network` (см. `Cargo.toml`
//! крейта). Без признака команды возвращают
//! [`crate::run::NetworkFeatureDisabled`].

mod args;
mod error;
mod run;

#[cfg(feature = "network")]
mod pull;
#[cfg(feature = "network")]
mod push;

#[cfg(feature = "network")]
pub use pull::{OutcomeCounts, compute_next_cursor_uid, parse_security, tally};
#[cfg(feature = "network")]
pub use push::send_outbox_record;

pub use args::{Args, SyncAction};
pub use error::SyncError;
pub use liveletters_output::CommandContext;
pub use run::run;

pub const COMMAND_NAME: &str = "sync";

pub fn summary() -> &'static str {
    "sync, sync pull / push"
}

pub fn crate_name() -> &'static str {
    "liveletters-lltt-sync"
}
