mod comments;
mod error;
mod extras;
mod metrics;
mod models;
mod outbox;
mod paths;
mod posts;
mod raw;
mod secret_bridge;
mod settings;
mod store;
mod subscriptions;
mod sync_state;

pub use error::StoreError;
pub use models::{
    BounceRecord, CommentRecord, DeferredEventRecord, DisplayNameRecord, MailSettingsRecord,
    OutboxDelivery, OutboxRecord, PendingSubscriptionRecord, PostRecord, RawEventRecord,
    RawMessageRecord, SubscriptionRecord, UserSettingsRecord,
};
pub use outbox::{decode_delivery, encode_delivery};
pub use paths::{EnvOverrides, StorePaths, resolve_data_dir, resolve_data_dir_from_env};
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
