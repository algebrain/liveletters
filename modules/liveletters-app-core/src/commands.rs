use liveletters_domain::{
    AccountId, Comment, CommentBody, CommentCreated, CommentCreatedFields, CommentEdited,
    CommentId, DomainError, EventId, Post, PostBody, PostCreated, PostHidden, PostId,
    ResourceAddress, ResourceId, Timestamp, Visibility,
};
use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolError, ProtocolIdentity, ProtocolMessage,
    encode_message,
};
use liveletters_store::{
    CommentRecord, MailSettingsRecord, OutboxDelivery, OutboxRecord, PostRecord, Store,
    UserSettingsRecord,
};
use liveletters_sync::{SyncEngine, SyncMessageOutcome};
use liveletters_utils::{
    email::{email_local_part, looks_like_email},
    text::require_non_blank,
};

use crate::{
    AppCoreError, AppSettings, DeferredReprocessingSummary, new_comment_id, new_post_id,
    post_created, post_hidden, subscription_requested, subscription_revoked, unix_millis_now,
};
use crate::{comment_created, comment_edited};

pub struct CreatePostCommand<'a> {
    pub profile_id: &'a str,
    pub post_id: &'a str,
    pub resource_id: &'a str,
    pub author_id: &'a str,
    pub created_at: u64,
    pub body: &'a str,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub publish: String,
}

pub struct CreatePostFromIdentityCommand<'a> {
    pub profile_id: &'a str,
    pub identity: &'a Identity,
    pub body: &'a str,
    pub visibility: Visibility,
}

pub struct CreateCommentCommand<'a> {
    pub profile_id: &'a str,
    pub comment_id: &'a str,
    pub post_id: &'a str,
    pub parent_comment_id: Option<&'a str>,
    pub author_id: &'a str,
    pub created_at: u64,
    pub body: &'a str,
    pub visibility: Visibility,
}

pub struct CreateCommentFromIdentityCommand<'a> {
    pub profile_id: &'a str,
    pub identity: &'a Identity,
    pub post_id: &'a str,
    pub parent_comment_id: Option<&'a str>,
    pub body: &'a str,
    pub visibility: Visibility,
}

pub struct HidePostCommand<'a> {
    pub profile_id: &'a str,
    pub post_id: &'a str,
    pub actor_id: &'a str,
    pub created_at: u64,
}

pub struct EditCommentCommand<'a> {
    pub profile_id: &'a str,
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
    pub smtp_security: &'a str,
    pub smtp_username: &'a str,
    pub smtp_password: &'a str,
    pub smtp_hello_domain: &'a str,
    pub imap_host: &'a str,
    pub imap_port: u16,
    pub imap_security: &'a str,
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
    let visibility = command.visibility;

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
        resource_email: post.resource_id().as_str().to_owned(),
        author_email: post.author_id().as_str().to_owned(),
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

    let record = store.get_user_settings_record(command.profile_id)?;
    let origin = protocol_origin(store, record.as_ref())?;

    let i18n = post_created(
        record.as_ref(),
        post.resource_id().as_str(),
        post.body().as_str(),
    );

    enqueue_message(
        store,
        command.profile_id,
        event.event_id().as_str(),
        "post_created",
        &i18n.subject,
        post.resource_id().as_str(),
        OutboxDelivery::ResourceSubscribers,
        ProtocolMessage::new(
            MessageEnvelope::new(
                "1",
                "post_created",
                post.resource_id().as_str(),
                event.event_id().as_str(),
            )?,
            origin,
            None,
            &i18n.body,
            DomainEventPayload::PostCreated {
                post_id: post.id().as_str().to_owned(),
                resource_id: post.resource_id().as_str().to_owned(),
                created_at: post.created_at().as_unix_seconds(),
                body: post.body().as_str().to_owned(),
                body_format: "plain".to_owned(),
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
    let visibility = command.visibility;

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
        author_email: comment.author_id().as_str().to_owned(),
        created_at: comment.created_at().as_unix_seconds(),
        body: comment.body().as_str().to_owned(),
        visibility: encode_visibility(comment.visibility()),
        hidden: comment.is_hidden(),
    })?;

    let event = CommentCreated::new(CommentCreatedFields {
        event_id: EventId::new(&format!("comment-created:{}", comment.id().as_str()))?,
        comment_id: comment.id().clone(),
        post_id: comment.post_id().clone(),
        parent_comment_id: comment.parent_comment_id().cloned(),
        resource_id: ResourceId::new(&post_record.resource_email)?,
        actor_id: comment.author_id().clone(),
        created_at: comment.created_at(),
        visibility: comment.visibility(),
    });

    let record = store.get_user_settings_record(command.profile_id)?;
    let author_name = display_author(store, record.as_ref())?;
    let origin = protocol_origin(store, record.as_ref())?;

    let i18n = comment_created(
        record.as_ref(),
        &author_name,
        comment.post_id().as_str(),
        comment.body().as_str(),
    );

    let delivery = if author_id.as_str() == post_record.author_email.as_str() {
        OutboxDelivery::ResourceSubscribers
    } else {
        OutboxDelivery::Direct(vec![event.resource_id().as_str().to_owned()])
    };

    enqueue_message(
        store,
        command.profile_id,
        event.event_id().as_str(),
        "comment_created",
        &i18n.subject,
        event.resource_id().as_str(),
        delivery,
        ProtocolMessage::new(
            MessageEnvelope::new(
                "1",
                "comment_created",
                event.resource_id().as_str(),
                event.event_id().as_str(),
            )?,
            origin,
            None,
            &i18n.body,
            DomainEventPayload::CommentCreated {
                comment_id: comment.id().as_str().to_owned(),
                post_id: comment.post_id().as_str().to_owned(),
                parent_comment_id: comment
                    .parent_comment_id()
                    .map(|parent_id| parent_id.as_str().to_owned()),
                resource_id: event.resource_id().as_str().to_owned(),
                created_at: comment.created_at().as_unix_seconds(),
                body: comment.body().as_str().to_owned(),
                body_format: "plain".to_owned(),
                visibility: encode_visibility(comment.visibility()),
            },
        )?,
    )?;

    Ok(CreateCommentResult { comment, event })
}

pub fn create_post_from_identity(
    store: &Store,
    command: CreatePostFromIdentityCommand<'_>,
) -> Result<CreatePostResult, AppCoreError> {
    let post_id = new_post_id()?;
    let created_at = unix_millis_now() / 1000;
    create_post(
        store,
        CreatePostCommand {
            profile_id: command.profile_id,
            post_id: &post_id,
            resource_id: &command.identity.publish,
            author_id: &command.identity.publish,
            created_at,
            body: command.body,
            visibility: command.visibility,
        },
    )
}

pub fn create_comment_from_identity(
    store: &Store,
    command: CreateCommentFromIdentityCommand<'_>,
) -> Result<CreateCommentResult, AppCoreError> {
    let comment_id = new_comment_id()?;
    let created_at = unix_millis_now() / 1000;
    create_comment(
        store,
        CreateCommentCommand {
            profile_id: command.profile_id,
            comment_id: &comment_id,
            post_id: command.post_id,
            parent_comment_id: command.parent_comment_id,
            author_id: &command.identity.publish,
            created_at,
            body: command.body,
            visibility: command.visibility,
        },
    )
}

pub fn hide_post(
    store: &Store,
    command: HidePostCommand<'_>,
) -> Result<HidePostResult, AppCoreError> {
    let record =
        store
            .get_post_record(command.post_id)?
            .ok_or_else(|| AppCoreError::PostNotFound {
                post_id: command.post_id.to_owned(),
            })?;

    let post = Post::new(
        PostId::new(&record.post_id)?,
        ResourceId::new(&record.resource_email)?,
        AccountId::new(&record.author_email)?,
        Timestamp::from_unix_seconds(record.created_at),
        PostBody::new(&record.body)?,
        decode_visibility(&record.visibility),
    )?
    .hide();

    store.save_post_record(&PostRecord {
        post_id: post.id().as_str().to_owned(),
        resource_email: post.resource_id().as_str().to_owned(),
        author_email: post.author_id().as_str().to_owned(),
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

    let record = store.get_user_settings_record(command.profile_id)?;
    let author_name = display_author(store, record.as_ref())?;
    let origin = protocol_origin(store, record.as_ref())?;

    let i18n = post_hidden(record.as_ref(), &author_name, event.post_id().as_str());

    enqueue_message(
        store,
        command.profile_id,
        event.event_id().as_str(),
        "post_hidden",
        &i18n.subject,
        event.resource_id().as_str(),
        OutboxDelivery::ResourceSubscribers,
        ProtocolMessage::new(
            MessageEnvelope::new(
                "1",
                "post_hidden",
                event.resource_id().as_str(),
                event.event_id().as_str(),
            )?,
            origin,
            None,
            &i18n.body,
            DomainEventPayload::PostHidden {
                post_id: event.post_id().as_str().to_owned(),
                resource_id: event.resource_id().as_str().to_owned(),
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
        AccountId::new(&record.author_email)?,
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
        author_email: comment.author_id().as_str().to_owned(),
        created_at: comment.created_at().as_unix_seconds(),
        body: comment.body().as_str().to_owned(),
        visibility: encode_visibility(comment.visibility()),
        hidden: comment.is_hidden(),
    })?;

    let event = CommentEdited::new(
        EventId::new(&format!("comment-edited:{}", comment.id().as_str()))?,
        comment.id().clone(),
        comment.post_id().clone(),
        ResourceId::new(&post_record.resource_email)?,
        AccountId::new(command.actor_id)?,
        Timestamp::from_unix_seconds(command.created_at),
    );

    let record = store.get_user_settings_record(command.profile_id)?;
    let author_name = display_author(store, record.as_ref())?;
    let origin = protocol_origin(store, record.as_ref())?;

    let i18n = comment_edited(
        record.as_ref(),
        &author_name,
        event.post_id().as_str(),
        comment.body().as_str(),
    );

    enqueue_message(
        store,
        command.profile_id,
        event.event_id().as_str(),
        "comment_edited",
        &i18n.subject,
        event.resource_id().as_str(),
        OutboxDelivery::ResourceSubscribers,
        ProtocolMessage::new(
            MessageEnvelope::new(
                "1",
                "comment_edited",
                event.resource_id().as_str(),
                event.event_id().as_str(),
            )?,
            origin,
            None,
            &i18n.body,
            DomainEventPayload::CommentEdited {
                comment_id: event.comment_id().as_str().to_owned(),
                post_id: event.post_id().as_str().to_owned(),
                resource_id: event.resource_id().as_str().to_owned(),
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
    let mut filtered = 0;
    let mut still_deferred = 0;

    for outcome in report.outcomes() {
        match outcome {
            SyncMessageOutcome::Applied { .. } => applied += 1,
            SyncMessageOutcome::Replay { .. } => replayed += 1,
            SyncMessageOutcome::Unauthorized { .. } => unauthorized += 1,
            SyncMessageOutcome::Invalid { .. } => invalid += 1,
            SyncMessageOutcome::Filtered { .. } => filtered += 1,
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
            filtered,
            still_deferred,
        ),
    })
}

pub fn save_settings(
    store: &Store,
    command: SaveSettingsCommand<'_>,
) -> Result<SaveSettingsResult, AppCoreError> {
    let existing_language = store
        .get_user_settings_record(default_profile_id())?
        .map(|record| record.language);

    validate_non_blank("nickname", command.nickname)?;
    validate_email(command.email_address)?;
    validate_non_blank("smtp_host", command.smtp_host)?;
    validate_port("smtp_port", command.smtp_port)?;
    validate_mail_security("smtp_security", command.smtp_security)?;
    validate_non_blank("smtp_username", command.smtp_username)?;
    validate_non_blank("smtp_password", command.smtp_password)?;
    validate_non_blank("imap_host", command.imap_host)?;
    validate_port("imap_port", command.imap_port)?;
    validate_mail_security("imap_security", command.imap_security)?;
    validate_non_blank("imap_username", command.imap_username)?;
    validate_non_blank("imap_password", command.imap_password)?;
    validate_non_blank("imap_mailbox", command.imap_mailbox)?;
    let smtp_hello_domain = infer_smtp_hello_domain(
        command.smtp_hello_domain,
        command.email_address,
        command.smtp_host,
    );

    let settings = AppSettings {
        nickname: command.nickname.trim().to_owned(),
        email_address: command.email_address.trim().to_owned(),
        avatar_url: command
            .avatar_url
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
        language: existing_language
            .unwrap_or_else(|| liveletters_i18n::detect_system_locale().as_str().to_owned()),
        smtp_host: command.smtp_host.trim().to_owned(),
        smtp_port: command.smtp_port,
        smtp_security: command.smtp_security.trim().to_owned(),
        smtp_username: command.smtp_username.trim().to_owned(),
        smtp_password: command.smtp_password.to_owned(),
        smtp_hello_domain,
        imap_host: command.imap_host.trim().to_owned(),
        imap_port: command.imap_port,
        imap_security: command.imap_security.trim().to_owned(),
        imap_username: command.imap_username.trim().to_owned(),
        imap_password: command.imap_password.to_owned(),
        imap_mailbox: command.imap_mailbox.trim().to_owned(),
        initial_lookback_days: 1,
        setup_completed: true,
    };

    // Атомарное сохранение идентичности: UPSERT в `authors` + UPSERT
    // в `user_settings` (последний хранит FK на authors.email).
    store.save_identity(
        default_profile_id(),
        settings.email_address.as_str(),
        settings.nickname.as_str(),
        settings.avatar_url.as_deref(),
        settings.language.as_str(),
        settings.setup_completed,
    )?;
    store.save_mail_settings_record(&MailSettingsRecord {
        profile_id: default_profile_id().to_owned(),
        smtp_host: settings.smtp_host.clone(),
        smtp_port: settings.smtp_port,
        smtp_security: settings.smtp_security.clone(),
        smtp_username: settings.smtp_username.clone(),
        smtp_password: settings.smtp_password.clone(),
        smtp_hello_domain: settings.smtp_hello_domain.clone(),
        imap_host: settings.imap_host.clone(),
        imap_port: settings.imap_port,
        imap_security: settings.imap_security.clone(),
        imap_username: settings.imap_username.clone(),
        imap_password: settings.imap_password.clone(),
        imap_mailbox: settings.imap_mailbox.clone(),
        initial_lookback_days: settings.initial_lookback_days,
    })?;

    Ok(SaveSettingsResult { settings })
}

fn display_author(
    store: &Store,
    record: Option<&UserSettingsRecord>,
) -> Result<String, AppCoreError> {
    let user = record.ok_or_else(|| {
        AppCoreError::ProfileIncomplete(
            "user_settings отсутствует; задайте профиль: lltt set nickname \"Имя\"".into(),
        )
    })?;
    let author = store
        .get_author(&user.author_email)
        .map_err(AppCoreError::Store)?
        .ok_or_else(|| {
            AppCoreError::ProfileIncomplete(format!(
                "user_settings.author_email={} отсутствует в authors",
                user.author_email
            ))
        })?;
    if author.nickname.is_empty() {
        // Фоллбэк: локальная часть e-mail до @.
        if let Some(local) = email_local_part(&author.email) {
            return Ok(local.to_owned());
        }
        return Err(AppCoreError::ProfileIncomplete(format!(
            "authors.nickname для {} пуст и e-mail не содержит локальной части",
            author.email
        )));
    }
    Ok(author.nickname)
}

fn protocol_origin(
    store: &Store,
    record: Option<&UserSettingsRecord>,
) -> Result<ProtocolIdentity, AppCoreError> {
    let user = record.ok_or_else(|| {
        AppCoreError::ProfileIncomplete(
            "user_settings отсутствует; задайте профиль: lltt set nickname \"Имя\"".into(),
        )
    })?;
    let author = store
        .get_author(&user.author_email)
        .map_err(AppCoreError::Store)?
        .ok_or_else(|| {
            AppCoreError::ProfileIncomplete(format!(
                "user_settings.author_email={} отсутствует в authors",
                user.author_email
            ))
        })?;
    let nickname = if author.nickname.trim().is_empty() {
        author.email.clone()
    } else {
        author.nickname
    };
    ProtocolIdentity::new(nickname, author.email)
        .map_err(|e| AppCoreError::Protocol(ProtocolError::InvalidIdentity(e.to_string())))
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
    require_non_blank(value, "settings").map_err(|_| AppCoreError::SettingsValidation {
        field: field.to_owned(),
        message: "must not be blank".to_owned(),
    })?;

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

    if !looks_like_email(value) {
        return Err(AppCoreError::SettingsValidation {
            field: "email_address".to_owned(),
            message: "must contain @".to_owned(),
        });
    }

    Ok(())
}

fn infer_smtp_hello_domain(hello_domain: &str, email_address: &str, smtp_host: &str) -> String {
    let trimmed = hello_domain.trim();
    if !trimmed.is_empty() {
        return trimmed.to_owned();
    }

    if let Some((_, domain)) = email_address.trim().split_once('@') {
        let domain = domain.trim();
        if !domain.is_empty() {
            return domain.to_owned();
        }
    }

    smtp_host.trim().to_owned()
}

fn validate_mail_security(field: &str, value: &str) -> Result<(), AppCoreError> {
    match value.trim() {
        "none" | "starttls" | "tls" => Ok(()),
        _ => Err(AppCoreError::SettingsValidation {
            field: field.to_owned(),
            message: "must be one of: none, starttls, tls".to_owned(),
        }),
    }
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

#[allow(clippy::too_many_arguments)]
fn enqueue_message(
    store: &Store,
    profile_id: &str,
    event_id: &str,
    event_type: &str,
    subject: &str,
    resource_id: &str,
    delivery: OutboxDelivery,
    message: ProtocolMessage,
) -> Result<(), AppCoreError> {
    if let OutboxDelivery::Direct(addrs) = &delivery
        && addrs.is_empty()
    {
        return Err(AppCoreError::InvalidDelivery(
            "прямая адресация требует непустой список адресатов".to_owned(),
        ));
    }

    // Домен для Message-ID берём из `user_settings.author_email` → `authors.email`
    // (часть после `@`). Если email пустой или не содержит `@`, используем
    // `liveletters.invalid` — DSN-сопоставление всё равно будет искать по
    // `event_id` внутри Message-ID, а не по самому домену.
    let user = store.get_user_settings_record(profile_id).ok().flatten();
    let author_email_for_msg = user
        .as_ref()
        .map(|u| u.author_email.clone())
        .unwrap_or_else(|| resource_id.to_owned());
    let domain = user
        .as_ref()
        .and_then(|u| u.author_email.split_once('@').map(|(_, d)| d.to_owned()))
        .unwrap_or_else(|| "liveletters.invalid".to_owned());
    let message_id = format!("<{event_id}@{domain}>");

    // Локализованное тело хранится в отдельной колонке outbox, а не
    // в JSON. JSON хранит только `envelope` + `payload` (поле
    // `human_readable_body` помечено `skip_serializing`).
    let human_readable_body = message.human_readable_body().map(str::to_owned);

    store.save_outbox_record(&OutboxRecord {
        event_id: event_id.to_owned(),
        event_type: event_type.to_owned(),
        author_email: author_email_for_msg,
        resource_email: Some(resource_id.to_owned()),
        delivery,
        message_body: encode_message(&message)?,
        message_id: Some(message_id),
        subject: if subject.is_empty() {
            None
        } else {
            Some(subject.to_owned())
        },
        human_readable_body,
    })?;

    Ok(())
}

impl From<DomainError> for AppCoreError {
    fn from(value: DomainError) -> Self {
        AppCoreError::Domain(value)
    }
}

pub struct SubscribeCommand<'a> {
    pub profile_id: &'a str,
    pub resource_address: &'a str,
    pub subscriber_delivery_address: &'a str,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeResult {
    pub event_id: String,
    pub resource_address: String,
    pub delivery_address: String,
}

pub fn subscribe(
    store: &Store,
    command: SubscribeCommand<'_>,
) -> Result<SubscribeResult, AppCoreError> {
    let resource = ResourceAddress::new(command.resource_address)?;
    let _subscriber = AccountId::new(command.subscriber_delivery_address)?;
    let delivery = ResourceAddress::new(command.subscriber_delivery_address)?;
    let resource_nickname = resource
        .as_str()
        .split('@')
        .next()
        .filter(|local| !local.is_empty())
        .unwrap_or(resource.as_str());
    store.save_author(
        resource.as_str(),
        resource_nickname,
        "subscription_requested",
    )?;

    // UPSERT-семантика: при существующей pending-подписке `requested_at`
    // сохраняется, обновляется только `last_attempt_at`. Это позволяет
    // повторно вызывать `lltt sub <addr>` пока подтверждение не пришло.
    store.save_pending_subscription(command.profile_id, resource.as_str(), command.created_at)?;
    store.update_pending_last_attempt(command.profile_id, resource.as_str(), command.created_at)?;

    let event_id_str = format!(
        "subscription:{}:{}:{}",
        resource.as_str(),
        command.subscriber_delivery_address,
        command.created_at,
    );
    let event_id = EventId::new(&event_id_str)?;
    let _created_at = Timestamp::from_unix_seconds(command.created_at);

    let user = store.get_user_settings_record(command.profile_id)?;
    let i18n = subscription_requested(user.as_ref(), delivery.as_str(), resource.as_str());

    let author = store
        .get_author(
            user.as_ref()
                .ok_or_else(|| AppCoreError::ProfileIncomplete("нет user_settings".into()))?
                .author_email
                .as_str(),
        )
        .map_err(AppCoreError::Store)?
        .ok_or_else(|| {
            AppCoreError::ProfileIncomplete("user_settings.author_email нет в authors".into())
        })?;
    let origin = ProtocolIdentity::new(author.nickname.clone(), author.email.clone())
        .map_err(|e| AppCoreError::Protocol(ProtocolError::InvalidIdentity(e.to_string())))?;

    let message = ProtocolMessage::new(
        MessageEnvelope::new(
            "1",
            "subscription_requested",
            resource.as_str(),
            event_id.as_str(),
        )?,
        origin,
        None,
        &i18n.body,
        DomainEventPayload::SubscriptionRequested {
            resource_id: resource.as_str().to_owned(),
            subscriber_delivery_address: delivery.as_str().to_owned(),
            created_at: command.created_at,
        },
    )?;

    enqueue_message(
        store,
        command.profile_id,
        event_id.as_str(),
        "subscription_requested",
        &i18n.subject,
        resource.as_str(),
        OutboxDelivery::Direct(vec![resource.as_str().to_owned()]),
        message,
    )?;

    Ok(SubscribeResult {
        event_id: event_id.as_str().to_owned(),
        resource_address: resource.as_str().to_owned(),
        delivery_address: delivery.as_str().to_owned(),
    })
}

pub struct UnsubscribeCommand<'a> {
    pub profile_id: &'a str,
    pub resource_address: &'a str,
    pub subscriber_delivery_address: &'a str,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubscribeResult {
    pub event_id: String,
    pub resource_address: String,
}

pub fn unsubscribe(
    store: &Store,
    command: UnsubscribeCommand<'_>,
) -> Result<UnsubscribeResult, AppCoreError> {
    let resource = ResourceAddress::new(command.resource_address)?;
    let _subscriber = AccountId::new(command.subscriber_delivery_address)?;
    let delivery = ResourceAddress::new(command.subscriber_delivery_address)?;
    let _created_at = Timestamp::from_unix_seconds(command.created_at);

    let event_id_str = format!(
        "unsubscription:{}:{}:{}",
        resource.as_str(),
        command.subscriber_delivery_address,
        command.created_at
    );
    let event_id = EventId::new(&event_id_str)?;

    let user = store.get_user_settings_record(command.profile_id)?;
    let origin = protocol_origin(store, user.as_ref())?;
    let i18n = subscription_revoked(user.as_ref(), delivery.as_str(), resource.as_str());

    let message = ProtocolMessage::new(
        MessageEnvelope::new(
            "1",
            "subscription_revoked",
            resource.as_str(),
            event_id.as_str(),
        )?,
        origin,
        None,
        &i18n.body,
        DomainEventPayload::SubscriptionRevoked {
            resource_id: resource.as_str().to_owned(),
            subscriber_delivery_address: delivery.as_str().to_owned(),
            created_at: command.created_at,
        },
    )?;

    enqueue_message(
        store,
        command.profile_id,
        event_id.as_str(),
        "subscription_revoked",
        &i18n.subject,
        resource.as_str(),
        OutboxDelivery::Direct(vec![resource.as_str().to_owned()]),
        message,
    )?;

    Ok(UnsubscribeResult {
        event_id: event_id.as_str().to_owned(),
        resource_address: resource.as_str().to_owned(),
    })
}
