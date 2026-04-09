mod dto;
mod errors;
mod reader;

pub use dto::{
    DeferredEventDiagnostic, DiagnosticsSnapshot, HealthStatus, OutboxDiagnostic,
    RawMessageDiagnostic, SyncHealth,
};
pub use errors::DiagnosticsError;
pub use reader::DiagnosticsReader;

pub fn crate_name() -> &'static str {
    "liveletters-diagnostics"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-diagnostics");
    }
}
