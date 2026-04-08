use crate::{
    AccountId, CommentId, EventId, PostId, ResourceId, Timestamp, Visibility,
};

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
    pub fn new(
        event_id: EventId,
        comment_id: CommentId,
        post_id: PostId,
        parent_comment_id: Option<CommentId>,
        resource_id: ResourceId,
        actor_id: AccountId,
        created_at: Timestamp,
        visibility: Visibility,
    ) -> Self {
        Self {
            event_id,
            comment_id,
            post_id,
            parent_comment_id,
            resource_id,
            actor_id,
            created_at,
            visibility,
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
