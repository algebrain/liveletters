use liveletters_store::{CommentRecord, OutboxRecord, PostRecord, Store};

use crate::{
    AppCoreError, AppSettings, BootstrapState, CommentSummary, HomeFeed, OutboxEntry,
    PendingOutbox, PostSummary, PostThread, decode_visibility_name,
};

pub struct GetHomeFeedQuery;

pub struct GetPostThreadQuery<'a> {
    pub post_id: &'a str,
}

pub struct GetPendingOutboxQuery;
pub struct GetBootstrapStateQuery;
pub struct GetSettingsQuery;

pub fn get_home_feed(store: &Store, _query: GetHomeFeedQuery) -> Result<HomeFeed, AppCoreError> {
    let posts = store
        .list_posts()?
        .into_iter()
        .map(post_summary_from_record)
        .collect();

    Ok(HomeFeed::new(posts))
}

pub fn get_post_thread(
    store: &Store,
    query: GetPostThreadQuery<'_>,
) -> Result<PostThread, AppCoreError> {
    let post = store
        .list_posts()?
        .into_iter()
        .find(|post| post.post_id == query.post_id)
        .ok_or_else(|| AppCoreError::PostNotFound {
            post_id: query.post_id.to_owned(),
        })?;

    let comments = store
        .list_comments_for_post(query.post_id)?
        .into_iter()
        .map(comment_summary_from_record)
        .collect();

    Ok(PostThread::new(post_summary_from_record(post), comments))
}

pub fn get_pending_outbox(
    store: &Store,
    _query: GetPendingOutboxQuery,
) -> Result<PendingOutbox, AppCoreError> {
    let entries = store
        .list_outbox_records()?
        .into_iter()
        .map(outbox_entry_from_record)
        .collect();

    Ok(PendingOutbox::new(entries))
}

pub fn get_bootstrap_state(
    store: &Store,
    _query: GetBootstrapStateQuery,
) -> Result<BootstrapState, AppCoreError> {
    let settings = store.get_user_settings_record(default_profile_id())?;

    Ok(BootstrapState::new(
        settings.map(|record| record.setup_completed).unwrap_or(false),
    ))
}

pub fn get_settings(
    store: &Store,
    _query: GetSettingsQuery,
) -> Result<AppSettings, AppCoreError> {
    let user = store.get_user_settings_record(default_profile_id())?;
    let mail = store.get_mail_settings_record(default_profile_id())?;
    let mut settings = AppSettings::empty();

    if let Some(user) = user {
        settings.nickname = user.nickname;
        settings.email_address = user.email_address;
        settings.avatar_url = user.avatar_url;
        settings.setup_completed = user.setup_completed;
    }

    if let Some(mail) = mail {
        settings.smtp_host = mail.smtp_host;
        settings.smtp_port = mail.smtp_port;
        settings.smtp_username = mail.smtp_username;
        settings.smtp_password = mail.smtp_password;
        settings.smtp_hello_domain = mail.smtp_hello_domain;
        settings.imap_host = mail.imap_host;
        settings.imap_port = mail.imap_port;
        settings.imap_username = mail.imap_username;
        settings.imap_password = mail.imap_password;
        settings.imap_mailbox = mail.imap_mailbox;
    }

    Ok(settings)
}

fn post_summary_from_record(record: PostRecord) -> PostSummary {
    PostSummary {
        post_id: record.post_id,
        resource_id: record.resource_id,
        author_id: record.author_id,
        created_at: record.created_at,
        body: record.body,
        visibility: decode_visibility_name(&record.visibility),
        hidden: record.hidden,
    }
}

fn comment_summary_from_record(record: CommentRecord) -> CommentSummary {
    CommentSummary {
        comment_id: record.comment_id,
        post_id: record.post_id,
        parent_comment_id: record.parent_comment_id,
        author_id: record.author_id,
        created_at: record.created_at,
        body: record.body,
        visibility: decode_visibility_name(&record.visibility),
        hidden: record.hidden,
    }
}

fn outbox_entry_from_record(record: OutboxRecord) -> OutboxEntry {
    OutboxEntry {
        event_id: record.event_id,
        event_type: record.event_type,
        resource_id: record.resource_id,
        message_body: record.message_body,
    }
}

fn default_profile_id() -> &'static str {
    "default"
}
