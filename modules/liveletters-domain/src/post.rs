use crate::{AccountId, DomainError, PostBody, PostId, ResourceId, Timestamp, Visibility};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    id: PostId,
    resource_id: ResourceId,
    author_id: AccountId,
    created_at: Timestamp,
    body: PostBody,
    visibility: Visibility,
    hidden: bool,
}

impl Post {
    pub fn new(
        id: PostId,
        resource_id: ResourceId,
        author_id: AccountId,
        created_at: Timestamp,
        body: PostBody,
        visibility: Visibility,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            resource_id,
            author_id,
            created_at,
            body,
            visibility,
            hidden: false,
        })
    }

    pub fn id(&self) -> &PostId {
        &self.id
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn body(&self) -> &PostBody {
        &self.body
    }

    pub fn author_id(&self) -> &AccountId {
        &self.author_id
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn hide(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn edit(mut self, body: PostBody) -> Self {
        self.body = body;
        self
    }
}
