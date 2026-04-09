mod error;
mod models;
mod paths;
mod store;

pub use error::StoreError;
pub use models::{
    CommentRecord, DeferredEventRecord, MailSettingsRecord, OutboxRecord, PostRecord,
    RawEventRecord, RawMessageRecord, UserSettingsRecord,
};
pub use paths::StorePaths;
pub use store::Store;

pub fn crate_name() -> &'static str {
    "liveletters-store"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-store");
    }
}
