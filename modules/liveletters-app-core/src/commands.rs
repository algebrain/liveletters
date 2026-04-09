use liveletters_domain::{
    AccountId, Comment, CommentBody, CommentCreated, CommentId, DomainError, EventId, Post,
    PostBody, PostCreated, PostHidden, PostId, ResourceId, Timestamp, Visibility, CommentEdited,
};
use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolMessage, encode_message,
};
use liveletters_sync::{SyncEngine, SyncMessageOutcome};
use liveletters_store::{
    CommentRecord, MailSettingsRecord, OutboxRecord, PostRecord, Store, UserSettingsRecord,
};

use crate::{AppCoreError, AppSettings, DeferredReprocessingSummary};

pub struct CreatePostCommand<'a> {
    pub post_id: &'a str,
    pub resource_id: &'a str,
    pub author_id: &'a str,
    pub created_at: u64,
    pub body: &'a str,
}

pub struct CreateCommentCommand<'a> {
    pub comment_id: &'a str,
    pub post_id: &'a str,
    pub parent_comment_id: Option<&'a str>,
    pub author_id: &'a str,
    pub created_at: u64,
    pub body: &'a str,
}

pub struct HidePostCommand<'a> {
    pub post_id: &'a str,
    pub actor_id: &'a str,
    pub created_at: u64,
}

pub struct EditCommentCommand<'a> {
    pub comment_id: &'a str,
    pub actor_id: &'a str,
    pub created_at: u64,
    pub body: &'a str,
}

pub struct ReprocessDeferredEventsCommand;

pub struct SaveSettingsCommand<'a> {
    pub nickname: &'a str,
    pub email_address: &'a str,
    pub avatar_url: Option<&'a str>,
    pub smtp_host: &'a str,
    pub smtp_port: u16,
    pub smtp_username: &'a str,
    pub smtp_password: &'a str,
    pub smtp_hello_domain: &'a str,
    pub imap_host: &'a str,
    pub imap_port: u16,
    pub imap_username: &'a str,
    pub imap_password: &'a str,
    pub imap_mailbox: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePostResult {
    post: Post,
    event: PostCreated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCommentResult {
    comment: Comment,
    event: CommentCreated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidePostResult {
    post: Post,
    event: PostHidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditCommentResult {
    comment: Comment,
    event: CommentEdited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReprocessDeferredEventsResult {
    summary: DeferredReprocessingSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSettingsResult {
    settings: AppSettings,
}

impl CreatePostResult {
    pub fn post(&self) -> &Post {
        &self.post
    }

    pub fn event(&self) -> &PostCreated {
        &self.event
    }
}

impl CreateCommentResult {
    pub fn comment(&self) -> &Comment {
        &self.comment
    }

    pub fn event(&self) -> &CommentCreated {
        &self.event
    }
}

impl HidePostResult {
    pub fn post(&self) -> &Post {
        &self.post
    }

    pub fn event(&self) -> &PostHidden {
        &self.event
    }
}

impl EditCommentResult {
    pub fn comment(&self) -> &Comment {
        &self.comment
    }

    pub fn event(&self) -> &CommentEdited {
        &self.event
    }
}

impl ReprocessDeferredEventsResult {
    pub fn summary(&self) -> &DeferredReprocessingSummary {
        &self.summary
    }
}

impl SaveSettingsResult {
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }
}

pub fn create_post(
    store: &Store,
    command: CreatePostCommand<'_>,
) -> Result<CreatePostResult, AppCoreError> {
    let post_id = PostId::new(command.post_id)?;
    let resource_id = ResourceId::new(command.resource_id)?;
    let author_id = AccountId::new(command.author_id)?;
    let created_at = Timestamp::from_unix_seconds(command.created_at);
    let body = PostBody::new(command.body)?;
    let visibility = Visibility::Public;

    let post = Post::new(
        post_id.clone(),
        resource_id.clone(),
        author_id.clone(),
        created_at,
        body,
        visibility,
    )?;

    store.save_post_record(&PostRecord {
        post_id: post.id().as_str().to_owned(),
        resource_id: post.resource_id().as_str().to_owned(),
        author_id: post.author_id().as_str().to_owned(),
        created_at: post.created_at().as_unix_seconds(),
        body: post.body().as_str().to_owned(),
        visibility: encode_visibility(post.visibility()),
        hidden: post.is_hidden(),
    })?;

    let event = PostCreated::new(
        EventId::new(&format!("post-created:{}", post.id().as_str()))?,
        post.id().clone(),
        post.resource_id().clone(),
        post.author_id().clone(),
        post.created_at(),
        post.visibility(),
    );

    enqueue_message(
        store,
        event.event_id().as_str(),
        "post_created",
        post.resource_id().as_str(),
        ProtocolMessage::new(
            MessageEnvelope::new("1", "post_created", post.resource_id().as_str(), event.event_id().as_str())?,
            "Новая запись в блоге",
            DomainEventPayload::PostCreated {
                post_id: post.id().as_str().to_owned(),
                resource_id: post.resource_id().as_str().to_owned(),
                actor_id: post.author_id().as_str().to_owned(),
                created_at: post.created_at().as_unix_seconds(),
                visibility: encode_visibility(post.visibility()),
            },
        )?,
    )?;

    Ok(CreatePostResult { post, event })
}

pub fn create_comment(
    store: &Store,
    command: CreateCommentCommand<'_>,
) -> Result<CreateCommentResult, AppCoreError> {
    let post_record = store.get_post_record(command.post_id)?;
    let Some(post_record) = post_record else {
        return Err(AppCoreError::PostNotFound {
            post_id: command.post_id.to_owned(),
        });
    };

    let comment_id = CommentId::new(command.comment_id)?;
    let post_id = PostId::new(command.post_id)?;
    let parent_comment_id = command.parent_comment_id.map(CommentId::new).transpose()?;
    let author_id = AccountId::new(command.author_id)?;
    let created_at = Timestamp::from_unix_seconds(command.created_at);
    let body = CommentBody::new(command.body)?;
    let visibility = Visibility::Public;

    let comment = Comment::new(
        comment_id.clone(),
        post_id.clone(),
        parent_comment_id.clone(),
        author_id.clone(),
        created_at,
        body,
        visibility,
    )?;

    store.save_comment_record(&CommentRecord {
        comment_id: comment.id().as_str().to_owned(),
        post_id: comment.post_id().as_str().to_owned(),
        parent_comment_id: comment
            .parent_comment_id()
            .map(|parent_id| parent_id.as_str().to_owned()),
        author_id: comment.author_id().as_str().to_owned(),
        created_at: comment.created_at().as_unix_seconds(),
        body: comment.body().as_str().to_owned(),
        visibility: encode_visibility(comment.visibility()),
        hidden: comment.is_hidden(),
    })?;

    let event = CommentCreated::new(
        EventId::new(&format!("comment-created:{}", comment.id().as_str()))?,
        comment.id().clone(),
        comment.post_id().clone(),
        comment.parent_comment_id().cloned(),
        ResourceId::new(&post_record.resource_id)?,
        comment.author_id().clone(),
        comment.created_at(),
        comment.visibility(),
    );

    enqueue_message(
        store,
        event.event_id().as_str(),
        "comment_created",
        event.resource_id().as_str(),
        ProtocolMessage::new(
            MessageEnvelope::new(
                "1",
                "comment_created",
                event.resource_id().as_str(),
                event.event_id().as_str(),
            )?,
            "Новый комментарий",
            DomainEventPayload::CommentCreated {
                comment_id: comment.id().as_str().to_owned(),
                post_id: comment.post_id().as_str().to_owned(),
                parent_comment_id: comment
                    .parent_comment_id()
                    .map(|parent_id| parent_id.as_str().to_owned()),
                resource_id: event.resource_id().as_str().to_owned(),
                actor_id: comment.author_id().as_str().to_owned(),
                created_at: comment.created_at().as_unix_seconds(),
                visibility: encode_visibility(comment.visibility()),
            },
        )?,
    )?;

    Ok(CreateCommentResult { comment, event })
}

pub fn hide_post(store: &Store, command: HidePostCommand<'_>) -> Result<HidePostResult, AppCoreError> {
    let record = store
        .get_post_record(command.post_id)?
        .ok_or_else(|| AppCoreError::PostNotFound {
            post_id: command.post_id.to_owned(),
        })?;

    let post = Post::new(
        PostId::new(&record.post_id)?,
        ResourceId::new(&record.resource_id)?,
        AccountId::new(&record.author_id)?,
        Timestamp::from_unix_seconds(record.created_at),
        PostBody::new(&record.body)?,
        decode_visibility(&record.visibility),
    )?
    .hide();

    store.save_post_record(&PostRecord {
        post_id: post.id().as_str().to_owned(),
        resource_id: post.resource_id().as_str().to_owned(),
        author_id: post.author_id().as_str().to_owned(),
        created_at: post.created_at().as_unix_seconds(),
        body: post.body().as_str().to_owned(),
        visibility: encode_visibility(post.visibility()),
        hidden: post.is_hidden(),
    })?;

    let event = PostHidden::new(
        EventId::new(&format!("post-hidden:{}", post.id().as_str()))?,
        post.id().clone(),
        post.resource_id().clone(),
        AccountId::new(command.actor_id)?,
        Timestamp::from_unix_seconds(command.created_at),
    );

    enqueue_message(
        store,
        event.event_id().as_str(),
        "post_hidden",
        event.resource_id().as_str(),
        ProtocolMessage::new(
            MessageEnvelope::new("1", "post_hidden", event.resource_id().as_str(), event.event_id().as_str())?,
            "Запись скрыта",
            DomainEventPayload::PostHidden {
                post_id: event.post_id().as_str().to_owned(),
                resource_id: event.resource_id().as_str().to_owned(),
                actor_id: event.actor_id().as_str().to_owned(),
                created_at: event.created_at().as_unix_seconds(),
            },
        )?,
    )?;

    Ok(HidePostResult { post, event })
}

pub fn edit_comment(
    store: &Store,
    command: EditCommentCommand<'_>,
) -> Result<EditCommentResult, AppCoreError> {
    let record = store
        .get_comment_record(command.comment_id)?
        .ok_or_else(|| AppCoreError::CommentNotFound {
            comment_id: command.comment_id.to_owned(),
        })?;

    let comment = Comment::new(
        CommentId::new(&record.comment_id)?,
        PostId::new(&record.post_id)?,
        record
            .parent_comment_id
            .as_deref()
            .map(CommentId::new)
            .transpose()?,
        AccountId::new(&record.author_id)?,
        Timestamp::from_unix_seconds(record.created_at),
        CommentBody::new(&record.body)?,
        decode_visibility(&record.visibility),
    )?
    .edit(CommentBody::new(command.body)?);

    let post_record = store
        .get_post_record(comment.post_id().as_str())?
        .ok_or_else(|| AppCoreError::PostNotFound {
            post_id: comment.post_id().as_str().to_owned(),
        })?;

    store.save_comment_record(&CommentRecord {
        comment_id: comment.id().as_str().to_owned(),
        post_id: comment.post_id().as_str().to_owned(),
        parent_comment_id: comment
            .parent_comment_id()
            .map(|parent_id| parent_id.as_str().to_owned()),
        author_id: comment.author_id().as_str().to_owned(),
        created_at: comment.created_at().as_unix_seconds(),
        body: comment.body().as_str().to_owned(),
        visibility: encode_visibility(comment.visibility()),
        hidden: comment.is_hidden(),
    })?;

    let event = CommentEdited::new(
        EventId::new(&format!("comment-edited:{}", comment.id().as_str()))?,
        comment.id().clone(),
        comment.post_id().clone(),
        ResourceId::new(&post_record.resource_id)?,
        AccountId::new(command.actor_id)?,
        Timestamp::from_unix_seconds(command.created_at),
    );

    enqueue_message(
        store,
        event.event_id().as_str(),
        "comment_edited",
        event.resource_id().as_str(),
        ProtocolMessage::new(
            MessageEnvelope::new(
                "1",
                "comment_edited",
                event.resource_id().as_str(),
                event.event_id().as_str(),
            )?,
            "Комментарий изменен",
            DomainEventPayload::CommentEdited {
                comment_id: event.comment_id().as_str().to_owned(),
                post_id: event.post_id().as_str().to_owned(),
                resource_id: event.resource_id().as_str().to_owned(),
                actor_id: event.actor_id().as_str().to_owned(),
                created_at: event.created_at().as_unix_seconds(),
                body: comment.body().as_str().to_owned(),
                visibility: encode_visibility(comment.visibility()),
            },
        )?,
    )?;

    Ok(EditCommentResult { comment, event })
}

pub fn reprocess_deferred_events(
    store: &Store,
    _command: ReprocessDeferredEventsCommand,
) -> Result<ReprocessDeferredEventsResult, AppCoreError> {
    let report = SyncEngine::new(store).reprocess_deferred()?;

    let mut applied = 0;
    let mut replayed = 0;
    let mut unauthorized = 0;
    let mut invalid = 0;
    let mut still_deferred = 0;

    for outcome in report.outcomes() {
        match outcome {
            SyncMessageOutcome::Applied { .. } => applied += 1,
            SyncMessageOutcome::Replay { .. } => replayed += 1,
            SyncMessageOutcome::Unauthorized { .. } => unauthorized += 1,
            SyncMessageOutcome::Invalid { .. } => invalid += 1,
            SyncMessageOutcome::Deferred { .. } => still_deferred += 1,
            SyncMessageOutcome::Duplicate { .. } | SyncMessageOutcome::Malformed { .. } => {}
        }
    }

    Ok(ReprocessDeferredEventsResult {
        summary: DeferredReprocessingSummary::new(
            applied,
            replayed,
            unauthorized,
            invalid,
            still_deferred,
        ),
    })
}

pub fn save_settings(
    store: &Store,
    command: SaveSettingsCommand<'_>,
) -> Result<SaveSettingsResult, AppCoreError> {
    validate_non_blank("nickname", command.nickname)?;
    validate_email(command.email_address)?;
    validate_non_blank("smtp_host", command.smtp_host)?;
    validate_port("smtp_port", command.smtp_port)?;
    validate_non_blank("smtp_username", command.smtp_username)?;
    validate_non_blank("smtp_password", command.smtp_password)?;
    validate_non_blank("smtp_hello_domain", command.smtp_hello_domain)?;
    validate_non_blank("imap_host", command.imap_host)?;
    validate_port("imap_port", command.imap_port)?;
    validate_non_blank("imap_username", command.imap_username)?;
    validate_non_blank("imap_password", command.imap_password)?;
    validate_non_blank("imap_mailbox", command.imap_mailbox)?;

    let settings = AppSettings {
        nickname: command.nickname.trim().to_owned(),
        email_address: command.email_address.trim().to_owned(),
        avatar_url: command
            .avatar_url
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
        smtp_host: command.smtp_host.trim().to_owned(),
        smtp_port: command.smtp_port,
        smtp_username: command.smtp_username.trim().to_owned(),
        smtp_password: command.smtp_password.to_owned(),
        smtp_hello_domain: command.smtp_hello_domain.trim().to_owned(),
        imap_host: command.imap_host.trim().to_owned(),
        imap_port: command.imap_port,
        imap_username: command.imap_username.trim().to_owned(),
        imap_password: command.imap_password.to_owned(),
        imap_mailbox: command.imap_mailbox.trim().to_owned(),
        setup_completed: true,
    };

    store.save_user_settings_record(&UserSettingsRecord {
        profile_id: default_profile_id().to_owned(),
        nickname: settings.nickname.clone(),
        email_address: settings.email_address.clone(),
        avatar_url: settings.avatar_url.clone(),
        setup_completed: settings.setup_completed,
    })?;
    store.save_mail_settings_record(&MailSettingsRecord {
        profile_id: default_profile_id().to_owned(),
        smtp_host: settings.smtp_host.clone(),
        smtp_port: settings.smtp_port,
        smtp_username: settings.smtp_username.clone(),
        smtp_password: settings.smtp_password.clone(),
        smtp_hello_domain: settings.smtp_hello_domain.clone(),
        imap_host: settings.imap_host.clone(),
        imap_port: settings.imap_port,
        imap_username: settings.imap_username.clone(),
        imap_password: settings.imap_password.clone(),
        imap_mailbox: settings.imap_mailbox.clone(),
    })?;

    Ok(SaveSettingsResult { settings })
}

fn encode_visibility(visibility: Visibility) -> String {
    match visibility {
        Visibility::Public => "public",
        Visibility::FriendsOnly => "friends_only",
        Visibility::MembersOnly => "members_only",
        Visibility::PrivateCommunity => "private_community",
    }
    .to_owned()
}

fn decode_visibility(value: &str) -> Visibility {
    match value {
        "friends_only" => Visibility::FriendsOnly,
        "members_only" => Visibility::MembersOnly,
        "private_community" => Visibility::PrivateCommunity,
        _ => Visibility::Public,
    }
}

fn default_profile_id() -> &'static str {
    "default"
}

fn validate_non_blank(field: &str, value: &str) -> Result<(), AppCoreError> {
    if value.trim().is_empty() {
        return Err(AppCoreError::SettingsValidation {
            field: field.to_owned(),
            message: "must not be blank".to_owned(),
        });
    }

    Ok(())
}

fn validate_email(value: &str) -> Result<(), AppCoreError> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(AppCoreError::SettingsValidation {
            field: "email_address".to_owned(),
            message: "must not be blank".to_owned(),
        });
    }

    if !trimmed.contains('@') {
        return Err(AppCoreError::SettingsValidation {
            field: "email_address".to_owned(),
            message: "must contain @".to_owned(),
        });
    }

    Ok(())
}

fn validate_port(field: &str, value: u16) -> Result<(), AppCoreError> {
    if value == 0 {
        return Err(AppCoreError::SettingsValidation {
            field: field.to_owned(),
            message: "must be greater than zero".to_owned(),
        });
    }

    Ok(())
}

fn enqueue_message(
    store: &Store,
    event_id: &str,
    event_type: &str,
    resource_id: &str,
    message: ProtocolMessage,
) -> Result<(), AppCoreError> {
    store.save_outbox_record(&OutboxRecord {
        event_id: event_id.to_owned(),
        event_type: event_type.to_owned(),
        resource_id: resource_id.to_owned(),
        message_body: encode_message(&message)?,
    })?;

    Ok(())
}

impl From<DomainError> for AppCoreError {
    fn from(value: DomainError) -> Self {
        AppCoreError::Domain(value)
    }
}
