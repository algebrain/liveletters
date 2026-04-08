use crate::{
    AccountId, CommentBody, CommentId, DomainError, PostId, Timestamp, Visibility,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    id: CommentId,
    post_id: PostId,
    parent_comment_id: Option<CommentId>,
    author_id: AccountId,
    created_at: Timestamp,
    body: CommentBody,
    visibility: Visibility,
    hidden: bool,
}

impl Comment {
    pub fn new(
        id: CommentId,
        post_id: PostId,
        parent_comment_id: Option<CommentId>,
        author_id: AccountId,
        created_at: Timestamp,
        body: CommentBody,
        visibility: Visibility,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            post_id,
            parent_comment_id,
            author_id,
            created_at,
            body,
            visibility,
            hidden: false,
        })
    }

    pub fn id(&self) -> &CommentId {
        &self.id
    }

    pub fn post_id(&self) -> &PostId {
        &self.post_id
    }

    pub fn parent_comment_id(&self) -> Option<&CommentId> {
        self.parent_comment_id.as_ref()
    }

    pub fn body(&self) -> &CommentBody {
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
}
