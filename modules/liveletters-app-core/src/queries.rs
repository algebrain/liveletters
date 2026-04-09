use liveletters_store::{CommentRecord, OutboxRecord, PostRecord, Store};

use crate::{
    AppCoreError, CommentSummary, HomeFeed, OutboxEntry, PendingOutbox, PostSummary, PostThread,
    decode_visibility_name,
};

pub struct GetHomeFeedQuery;

pub struct GetPostThreadQuery<'a> {
    pub post_id: &'a str,
}

pub struct GetPendingOutboxQuery;

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
