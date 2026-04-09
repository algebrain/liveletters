use liveletters_mail::{ReceivedEmail, decode_protocol_message, extract_liveletters_parts, parse_email};
use liveletters_protocol::DomainEventPayload;
use liveletters_store::{
    CommentRecord, DeferredEventRecord, PostRecord, RawEventRecord, RawMessageRecord, Store,
    StoreError,
};

use crate::{SyncMessageOutcome, SyncReport, SyncError};

pub struct SyncEngine<'a> {
    store: &'a Store,
}

impl<'a> SyncEngine<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn ingest_batch(&self, messages: Vec<ReceivedEmail>) -> Result<SyncReport, SyncError> {
        let mut outcomes = Vec::new();

        for message in messages {
            outcomes.push(self.ingest_one(message)?);
        }

        Ok(SyncReport::new(outcomes))
    }

    fn ingest_one(&self, message: ReceivedEmail) -> Result<SyncMessageOutcome, SyncError> {
        let parsed = match parse_email(&message.raw_message) {
            Ok(parsed) => parsed,
            Err(error) => {
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "malformed".into(),
                })?;
                return Ok(SyncMessageOutcome::Malformed {
                    message_id: message.message_id,
                    reason: format!("{error:?}"),
                });
            }
        };

        let parts = match extract_liveletters_parts(&parsed) {
            Ok(parts) => parts,
            Err(error) => {
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "malformed".into(),
                })?;
                return Ok(SyncMessageOutcome::Malformed {
                    message_id: message.message_id,
                    reason: format!("{error:?}"),
                });
            }
        };

        let protocol_message = match decode_protocol_message(parts.technical_body()) {
            Ok(protocol_message) => protocol_message,
            Err(error) => {
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "malformed".into(),
                })?;
                return Ok(SyncMessageOutcome::Malformed {
                    message_id: message.message_id,
                    reason: format!("{error:?}"),
                });
            }
        };

        let event_id = protocol_message.envelope().event_id().to_owned();
        if self.store.has_raw_event(&event_id)? {
            self.store.save_raw_message_record(&RawMessageRecord {
                message_id: message.message_id.clone(),
                raw_message: message.raw_message,
                status: "duplicate".into(),
            })?;
            return Ok(SyncMessageOutcome::Duplicate {
                message_id: message.message_id,
                event_id,
            });
        }

        let payload_json =
            serde_json::to_string(protocol_message.payload()).map_err(SyncError::SerializePayload)?;
        self.store.save_raw_event_record(&RawEventRecord {
            event_id: event_id.clone(),
            event_type: protocol_message.envelope().event_type().to_owned(),
            resource_id: protocol_message.envelope().resource_id().to_owned(),
            payload_json: payload_json.clone(),
        })?;

        let apply_result = self.apply_payload(protocol_message.payload(), protocol_message.envelope().resource_id());

        match apply_result {
            Ok(()) => {
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "applied".into(),
                })?;
                Ok(SyncMessageOutcome::Applied {
                    message_id: message.message_id,
                    event_id,
                })
            }
            Err(ApplyEventError::Deferred(reason)) => {
                self.store.save_deferred_event_record(&DeferredEventRecord {
                    event_id: event_id.clone(),
                    event_type: protocol_message.envelope().event_type().to_owned(),
                    reason: reason.clone(),
                    payload_json,
                })?;
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "deferred".into(),
                })?;
                Ok(SyncMessageOutcome::Deferred {
                    message_id: message.message_id,
                    event_id,
                    reason,
                })
            }
            Err(ApplyEventError::Store(error)) => Err(SyncError::Store(error)),
        }
    }

    fn apply_payload(
        &self,
        payload: &DomainEventPayload,
        resource_id: &str,
    ) -> Result<(), ApplyEventError> {
        match payload {
            DomainEventPayload::PostCreated {
                post_id,
                actor_id,
                created_at,
                visibility,
                ..
            } => self
                .store
                .save_post_record(&PostRecord {
                    post_id: post_id.clone(),
                    resource_id: resource_id.to_owned(),
                    author_id: actor_id.clone(),
                    created_at: *created_at,
                    body: "Imported post".into(),
                    visibility: visibility.clone(),
                    hidden: false,
                })
                .map_err(ApplyEventError::Store),
            DomainEventPayload::CommentCreated {
                comment_id,
                post_id,
                parent_comment_id,
                actor_id,
                created_at,
                visibility,
                ..
            } => {
                if self
                    .store
                    .get_post_record(post_id)
                    .map_err(ApplyEventError::Store)?
                    .is_none()
                {
                    return Err(ApplyEventError::Deferred("missing_post".into()));
                }

                self.store
                    .save_comment_record(&CommentRecord {
                        comment_id: comment_id.clone(),
                        post_id: post_id.clone(),
                        parent_comment_id: parent_comment_id.clone(),
                        author_id: actor_id.clone(),
                        created_at: *created_at,
                        body: "Imported comment".into(),
                        visibility: visibility.clone(),
                        hidden: false,
                    })
                    .map_err(ApplyEventError::Store)
            }
            DomainEventPayload::PostHidden { post_id, .. } => {
                let Some(existing) = self
                    .store
                    .get_post_record(post_id)
                    .map_err(ApplyEventError::Store)?
                else {
                    return Err(ApplyEventError::Deferred("missing_post".into()));
                };

                self.store
                    .save_post_record(&PostRecord {
                        hidden: true,
                        ..existing
                    })
                    .map_err(ApplyEventError::Store)
            }
            DomainEventPayload::CommentEdited {
                comment_id,
                body,
                ..
            } => {
                let Some(existing) = self
                    .store
                    .get_comment_record(comment_id)
                    .map_err(ApplyEventError::Store)?
                else {
                    return Err(ApplyEventError::Deferred("missing_comment".into()));
                };

                self.store
                    .save_comment_record(&CommentRecord {
                        body: body.clone(),
                        ..existing
                    })
                    .map_err(ApplyEventError::Store)
            }
        }
    }
}

enum ApplyEventError {
    Deferred(String),
    Store(StoreError),
}
