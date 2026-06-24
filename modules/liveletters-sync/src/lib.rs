mod engine;
mod errors;
mod limits;
mod reporting;

pub use engine::SyncEngine;
pub use errors::SyncError;
pub use limits::{IngestLimits, RetentionPolicy};
pub use reporting::{SyncMessageOutcome, SyncReport};

pub fn crate_name() -> &'static str {
    "liveletters-sync"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-sync");
    }
}
