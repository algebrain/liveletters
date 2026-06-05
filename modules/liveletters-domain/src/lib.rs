mod address;
mod comment;
mod errors;
mod events;
mod ids;
mod post;
mod subscription;
mod values;
mod visibility;

pub use address::ResourceAddress;
pub use comment::Comment;
pub use errors::DomainError;
pub use events::{
    CommentCreated, CommentCreatedFields, CommentEdited, CommentHidden, PostCreated, PostHidden,
    SubscriptionAction, SubscriptionChanged,
};
pub use ids::{AccountId, CommentId, EventId, PostId, ResourceId};
pub use post::Post;
pub use subscription::Subscription;
pub use values::{CommentBody, PostBody, Timestamp};
pub use visibility::Visibility;

pub fn crate_name() -> &'static str {
    "liveletters-domain"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-domain");
    }
}
