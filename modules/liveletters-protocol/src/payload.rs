use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DomainEventPayload {
    PostCreated {
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        visibility: String,
    },
    CommentCreated {
        comment_id: String,
        post_id: String,
        parent_comment_id: Option<String>,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        visibility: String,
    },
    PostHidden {
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
    },
    CommentEdited {
        comment_id: String,
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        body: String,
        visibility: String,
    },
    SubscriptionChanged {
        resource_address: String,
        subscriber_account_id: String,
        subscriber_delivery_address: String,
        action: String,
        created_at: u64,
    },
}
