use crate::{
    AccountId, CommentId, EventId, PostId, ResourceAddress, ResourceId, Timestamp, Visibility,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionAction {
    Subscribe,
    Unsubscribe,
}

impl SubscriptionAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Subscribe => "subscribe",
            Self::Unsubscribe => "unsubscribe",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionChanged {
    event_id: EventId,
    resource_address: ResourceAddress,
    subscriber_account_id: AccountId,
    subscriber_delivery_address: ResourceAddress,
    action: SubscriptionAction,
    created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostCreated {
    event_id: EventId,
    post_id: PostId,
    resource_id: ResourceId,
    actor_id: AccountId,
    created_at: Timestamp,
    visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentCreated {
    event_id: EventId,
    comment_id: CommentId,
    post_id: PostId,
    parent_comment_id: Option<CommentId>,
    resource_id: ResourceId,
    actor_id: AccountId,
    created_at: Timestamp,
    visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentCreatedFields {
    pub event_id: EventId,
    pub comment_id: CommentId,
    pub post_id: PostId,
    pub parent_comment_id: Option<CommentId>,
    pub resource_id: ResourceId,
    pub actor_id: AccountId,
    pub created_at: Timestamp,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostHidden {
    event_id: EventId,
    post_id: PostId,
    resource_id: ResourceId,
    actor_id: AccountId,
    created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentEdited {
    event_id: EventId,
    comment_id: CommentId,
    post_id: PostId,
    resource_id: ResourceId,
    actor_id: AccountId,
    created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentHidden {
    event_id: EventId,
    comment_id: CommentId,
    post_id: PostId,
    resource_id: ResourceId,
    actor_id: AccountId,
    created_at: Timestamp,
}

impl PostCreated {
    pub fn new(
        event_id: EventId,
        post_id: PostId,
        resource_id: ResourceId,
        actor_id: AccountId,
        created_at: Timestamp,
        visibility: Visibility,
    ) -> Self {
        Self {
            event_id,
            post_id,
            resource_id,
            actor_id,
            created_at,
            visibility,
        }
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn post_id(&self) -> &PostId {
        &self.post_id
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn actor_id(&self) -> &AccountId {
        &self.actor_id
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
}

impl CommentCreated {
    pub fn new(fields: CommentCreatedFields) -> Self {
        Self {
            event_id: fields.event_id,
            comment_id: fields.comment_id,
            post_id: fields.post_id,
            parent_comment_id: fields.parent_comment_id,
            resource_id: fields.resource_id,
            actor_id: fields.actor_id,
            created_at: fields.created_at,
            visibility: fields.visibility,
        }
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn comment_id(&self) -> &CommentId {
        &self.comment_id
    }

    pub fn post_id(&self) -> &PostId {
        &self.post_id
    }

    pub fn parent_comment_id(&self) -> Option<&CommentId> {
        self.parent_comment_id.as_ref()
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn actor_id(&self) -> &AccountId {
        &self.actor_id
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
}

impl PostHidden {
    pub fn new(
        event_id: EventId,
        post_id: PostId,
        resource_id: ResourceId,
        actor_id: AccountId,
        created_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            post_id,
            resource_id,
            actor_id,
            created_at,
        }
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn post_id(&self) -> &PostId {
        &self.post_id
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn actor_id(&self) -> &AccountId {
        &self.actor_id
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
}

impl CommentEdited {
    pub fn new(
        event_id: EventId,
        comment_id: CommentId,
        post_id: PostId,
        resource_id: ResourceId,
        actor_id: AccountId,
        created_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            comment_id,
            post_id,
            resource_id,
            actor_id,
            created_at,
        }
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn comment_id(&self) -> &CommentId {
        &self.comment_id
    }

    pub fn post_id(&self) -> &PostId {
        &self.post_id
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn actor_id(&self) -> &AccountId {
        &self.actor_id
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
}

impl CommentHidden {
    pub fn new(
        event_id: EventId,
        comment_id: CommentId,
        post_id: PostId,
        resource_id: ResourceId,
        actor_id: AccountId,
        created_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            comment_id,
            post_id,
            resource_id,
            actor_id,
            created_at,
        }
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn comment_id(&self) -> &CommentId {
        &self.comment_id
    }

    pub fn post_id(&self) -> &PostId {
        &self.post_id
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn actor_id(&self) -> &AccountId {
        &self.actor_id
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
}

impl SubscriptionChanged {
    pub fn new(
        event_id: EventId,
        resource_address: ResourceAddress,
        subscriber_account_id: AccountId,
        subscriber_delivery_address: ResourceAddress,
        action: SubscriptionAction,
        created_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            resource_address,
            subscriber_account_id,
            subscriber_delivery_address,
            action,
            created_at,
        }
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn resource_address(&self) -> &ResourceAddress {
        &self.resource_address
    }

    pub fn subscriber_account_id(&self) -> &AccountId {
        &self.subscriber_account_id
    }

    pub fn subscriber_delivery_address(&self) -> &ResourceAddress {
        &self.subscriber_delivery_address
    }

    pub fn action(&self) -> SubscriptionAction {
        self.action
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
}
