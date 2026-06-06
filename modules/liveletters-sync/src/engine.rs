use liveletters_mail::{
    ReceivedEmail, decode_protocol_message, extract_liveletters_parts, parse_email,
};
use liveletters_protocol::DomainEventPayload;
use liveletters_store::{
    CommentRecord, DeferredEventRecord, PostRecord, RawEventRecord, RawMessageRecord, Store,
    StoreError, SubscriptionRecord,
};

use crate::{SyncError, SyncMessageOutcome, SyncReport};

pub struct SyncEngine<'a> {
    store: &'a Store,
    identity_filter: Option<IdentityFilter<'a>>,
}

struct IdentityFilter<'a> {
    own_address: &'a str,
    subscribed: Vec<&'a str>,
}

impl<'a> SyncEngine<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self {
            store,
            identity_filter: None,
        }
    }

    pub fn new_with_identity(
        store: &'a Store,
        own_address: &'a str,
        subscribed: &'a [String],
    ) -> Self {
        let subscribed_refs: Vec<&str> = subscribed.iter().map(String::as_str).collect();
        Self {
            store,
            identity_filter: Some(IdentityFilter {
                own_address,
                subscribed: subscribed_refs,
            }),
        }
    }

    pub fn ingest_batch(&self, messages: Vec<ReceivedEmail>) -> Result<SyncReport, SyncError> {
        let mut outcomes = Vec::new();

        for message in messages {
            outcomes.push(self.ingest_one(message)?);
        }

        Ok(SyncReport::new(outcomes))
    }

    pub fn reprocess_deferred(&self) -> Result<SyncReport, SyncError> {
        let deferred_records = self.store.list_deferred_event_records()?;
        let mut outcomes = Vec::new();

        for record in deferred_records {
            let payload: DomainEventPayload = serde_json::from_str(&record.payload_json)
                .map_err(SyncError::DeserializePayload)?;

            let outcome = match self.apply_payload(&payload, infer_resource_id(&payload)) {
                Ok(()) => {
                    self.store.delete_deferred_event_record(&record.event_id)?;
                    self.store.save_raw_event_record(&RawEventRecord {
                        event_id: record.event_id.clone(),
                        event_type: record.event_type.clone(),
                        resource_id: infer_resource_id(&payload).to_owned(),
                        payload_json: record.payload_json,
                        apply_status: "applied".into(),
                        failure_reason: None,
                    })?;
                    SyncMessageOutcome::Applied {
                        message_id: format!("deferred:{}", record.event_id),
                        event_id: record.event_id,
                    }
                }
                Err(ApplyEventError::Deferred(reason)) => SyncMessageOutcome::Deferred {
                    message_id: format!("deferred:{}", record.event_id),
                    event_id: record.event_id,
                    reason,
                },
                Err(ApplyEventError::Replay(reason)) => {
                    self.store.delete_deferred_event_record(&record.event_id)?;
                    self.store.save_raw_event_record(&RawEventRecord {
                        event_id: record.event_id.clone(),
                        event_type: record.event_type.clone(),
                        resource_id: infer_resource_id(&payload).to_owned(),
                        payload_json: record.payload_json,
                        apply_status: "replay".into(),
                        failure_reason: Some(reason.clone()),
                    })?;
                    SyncMessageOutcome::Replay {
                        message_id: format!("deferred:{}", record.event_id),
                        event_id: record.event_id,
                        reason,
                    }
                }
                Err(ApplyEventError::Unauthorized(reason)) => {
                    self.store.delete_deferred_event_record(&record.event_id)?;
                    self.store.save_raw_event_record(&RawEventRecord {
                        event_id: record.event_id.clone(),
                        event_type: record.event_type.clone(),
                        resource_id: infer_resource_id(&payload).to_owned(),
                        payload_json: record.payload_json,
                        apply_status: "unauthorized".into(),
                        failure_reason: Some(reason.clone()),
                    })?;
                    SyncMessageOutcome::Unauthorized {
                        message_id: format!("deferred:{}", record.event_id),
                        event_id: record.event_id,
                        reason,
                    }
                }
                Err(ApplyEventError::Invalid(reason)) => {
                    self.store.delete_deferred_event_record(&record.event_id)?;
                    self.store.save_raw_event_record(&RawEventRecord {
                        event_id: record.event_id.clone(),
                        event_type: record.event_type.clone(),
                        resource_id: infer_resource_id(&payload).to_owned(),
                        payload_json: record.payload_json,
                        apply_status: "invalid".into(),
                        failure_reason: Some(reason.clone()),
                    })?;
                    SyncMessageOutcome::Invalid {
                        message_id: format!("deferred:{}", record.event_id),
                        event_id: record.event_id,
                        reason,
                    }
                }
                Err(ApplyEventError::Store(error)) => return Err(SyncError::Store(error)),
                Err(ApplyEventError::Filtered(reason)) => SyncMessageOutcome::Filtered {
                    message_id: format!("deferred:{}", record.event_id),
                    event_id: record.event_id,
                    reason,
                },
            };

            outcomes.push(outcome);
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

        if let Err(reason) = validate_protocol_message(&protocol_message) {
            self.store.save_raw_event_record(&RawEventRecord {
                event_id: protocol_message.envelope().event_id().to_owned(),
                event_type: protocol_message.envelope().event_type().to_owned(),
                resource_id: protocol_message.envelope().resource_id().to_owned(),
                payload_json: serde_json::to_string(protocol_message.payload())
                    .map_err(SyncError::SerializePayload)?,
                apply_status: "invalid".into(),
                failure_reason: Some(reason.clone()),
            })?;
            self.store.save_raw_message_record(&RawMessageRecord {
                message_id: message.message_id.clone(),
                raw_message: message.raw_message,
                status: "invalid".into(),
            })?;
            return Ok(SyncMessageOutcome::Invalid {
                message_id: message.message_id,
                event_id: protocol_message.envelope().event_id().to_owned(),
                reason,
            });
        }

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

        let payload_json = serde_json::to_string(protocol_message.payload())
            .map_err(SyncError::SerializePayload)?;
        self.store.save_raw_event_record(&RawEventRecord {
            event_id: event_id.clone(),
            event_type: protocol_message.envelope().event_type().to_owned(),
            resource_id: protocol_message.envelope().resource_id().to_owned(),
            payload_json: payload_json.clone(),
            apply_status: "pending".into(),
            failure_reason: None,
        })?;

        let apply_result = self.apply_payload(
            protocol_message.payload(),
            protocol_message.envelope().resource_id(),
        );

        match apply_result {
            Ok(()) => {
                self.store.save_raw_event_record(&RawEventRecord {
                    event_id: event_id.clone(),
                    event_type: protocol_message.envelope().event_type().to_owned(),
                    resource_id: protocol_message.envelope().resource_id().to_owned(),
                    payload_json,
                    apply_status: "applied".into(),
                    failure_reason: None,
                })?;
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
                self.store
                    .save_deferred_event_record(&DeferredEventRecord {
                        event_id: event_id.clone(),
                        event_type: protocol_message.envelope().event_type().to_owned(),
                        reason: reason.clone(),
                        payload_json: payload_json.clone(),
                    })?;
                self.store.save_raw_event_record(&RawEventRecord {
                    event_id: event_id.clone(),
                    event_type: protocol_message.envelope().event_type().to_owned(),
                    resource_id: protocol_message.envelope().resource_id().to_owned(),
                    payload_json: payload_json.clone(),
                    apply_status: "deferred".into(),
                    failure_reason: Some(reason.clone()),
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
            Err(ApplyEventError::Replay(reason)) => {
                self.store.save_raw_event_record(&RawEventRecord {
                    event_id: event_id.clone(),
                    event_type: protocol_message.envelope().event_type().to_owned(),
                    resource_id: protocol_message.envelope().resource_id().to_owned(),
                    payload_json: payload_json.clone(),
                    apply_status: "replay".into(),
                    failure_reason: Some(reason.clone()),
                })?;
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "replay".into(),
                })?;
                Ok(SyncMessageOutcome::Replay {
                    message_id: message.message_id,
                    event_id,
                    reason,
                })
            }
            Err(ApplyEventError::Unauthorized(reason)) => {
                self.store.save_raw_event_record(&RawEventRecord {
                    event_id: event_id.clone(),
                    event_type: protocol_message.envelope().event_type().to_owned(),
                    resource_id: protocol_message.envelope().resource_id().to_owned(),
                    payload_json: payload_json.clone(),
                    apply_status: "unauthorized".into(),
                    failure_reason: Some(reason.clone()),
                })?;
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "unauthorized".into(),
                })?;
                Ok(SyncMessageOutcome::Unauthorized {
                    message_id: message.message_id,
                    event_id,
                    reason,
                })
            }
            Err(ApplyEventError::Invalid(reason)) => {
                self.store.save_raw_event_record(&RawEventRecord {
                    event_id: event_id.clone(),
                    event_type: protocol_message.envelope().event_type().to_owned(),
                    resource_id: protocol_message.envelope().resource_id().to_owned(),
                    payload_json,
                    apply_status: "invalid".into(),
                    failure_reason: Some(reason.clone()),
                })?;
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "invalid".into(),
                })?;
                Ok(SyncMessageOutcome::Invalid {
                    message_id: message.message_id,
                    event_id,
                    reason,
                })
            }
            Err(ApplyEventError::Filtered(reason)) => {
                self.store.save_raw_event_record(&RawEventRecord {
                    event_id: event_id.clone(),
                    event_type: protocol_message.envelope().event_type().to_owned(),
                    resource_id: protocol_message.envelope().resource_id().to_owned(),
                    payload_json,
                    apply_status: "filtered".into(),
                    failure_reason: Some(reason.clone()),
                })?;
                self.store.save_raw_message_record(&RawMessageRecord {
                    message_id: message.message_id.clone(),
                    raw_message: message.raw_message,
                    status: "filtered".into(),
                })?;
                Ok(SyncMessageOutcome::Filtered {
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
        if let Some(filter) = &self.identity_filter
            && matches!(
                payload,
                DomainEventPayload::PostCreated { .. } | DomainEventPayload::CommentCreated { .. }
            )
        {
            let allowed =
                resource_id == filter.own_address || filter.subscribed.contains(&resource_id);
            if !allowed {
                return Err(ApplyEventError::Filtered("not_subscribed".into()));
            }
        }

        match payload {
            DomainEventPayload::PostCreated {
                post_id,
                actor_id,
                created_at,
                body,
                visibility,
                ..
            } => {
                if self
                    .store
                    .get_post_record(post_id)
                    .map_err(ApplyEventError::Store)?
                    .is_some()
                {
                    return Err(ApplyEventError::Replay("post_already_exists".into()));
                }

                self.store
                    .save_post_record(&PostRecord {
                        post_id: post_id.clone(),
                        resource_id: resource_id.to_owned(),
                        author_id: actor_id.clone(),
                        created_at: *created_at,
                        body: body.clone(),
                        visibility: visibility.clone(),
                        hidden: false,
                    })
                    .map_err(ApplyEventError::Store)
            }
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

                if self
                    .store
                    .get_comment_record(comment_id)
                    .map_err(ApplyEventError::Store)?
                    .is_some()
                {
                    return Err(ApplyEventError::Replay("comment_already_exists".into()));
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

                if existing.hidden {
                    return Err(ApplyEventError::Replay("post_already_hidden".into()));
                }

                if let DomainEventPayload::PostHidden { actor_id, .. } = payload
                    && existing.author_id != *actor_id
                {
                    return Err(ApplyEventError::Unauthorized(
                        "actor_cannot_hide_post".into(),
                    ));
                }

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
                visibility,
                ..
            } => {
                let Some(existing) = self
                    .store
                    .get_comment_record(comment_id)
                    .map_err(ApplyEventError::Store)?
                else {
                    return Err(ApplyEventError::Deferred("missing_comment".into()));
                };

                if let DomainEventPayload::CommentEdited {
                    actor_id,
                    body,
                    visibility,
                    ..
                } = payload
                {
                    if existing.author_id != *actor_id {
                        return Err(ApplyEventError::Unauthorized(
                            "actor_cannot_edit_comment".into(),
                        ));
                    }

                    if body.trim().is_empty() {
                        return Err(ApplyEventError::Invalid("blank_comment_body".into()));
                    }

                    if visibility.trim().is_empty() {
                        return Err(ApplyEventError::Invalid("blank_visibility".into()));
                    }

                    if existing.body == *body && existing.visibility == *visibility {
                        return Err(ApplyEventError::Replay(
                            "comment_edit_already_applied".into(),
                        ));
                    }
                }

                self.store
                    .save_comment_record(&CommentRecord {
                        body: body.clone(),
                        visibility: visibility.clone(),
                        ..existing
                    })
                    .map_err(ApplyEventError::Store)
            }
            DomainEventPayload::SubscriptionChanged {
                resource_address,
                subscriber_delivery_address,
                active,
                ..
            } => {
                let record = SubscriptionRecord {
                    resource_address: resource_address.clone(),
                    subscriber_delivery_address: subscriber_delivery_address.clone(),
                };
                if *active {
                    self.store
                        .save_subscription(&record)
                        .map_err(ApplyEventError::Store)
                } else {
                    let _ = self
                        .store
                        .delete_subscription(resource_address, subscriber_delivery_address)
                        .map_err(ApplyEventError::Store)?;
                    Ok(())
                }
            }
        }
    }
}

enum ApplyEventError {
    Deferred(String),
    Replay(String),
    Unauthorized(String),
    Invalid(String),
    Filtered(String),
    Store(StoreError),
}

fn validate_protocol_message(
    protocol_message: &liveletters_protocol::ProtocolMessage,
) -> Result<(), String> {
    let envelope = protocol_message.envelope();
    let payload = protocol_message.payload();

    let payload_resource_id = infer_resource_id(payload);
    if envelope.resource_id() != payload_resource_id {
        return Err("resource_id_mismatch".into());
    }

    if envelope.event_type() != infer_event_type(payload) {
        return Err("event_type_mismatch".into());
    }

    let actor_id = infer_actor_id(payload);
    if matches!(
        payload,
        DomainEventPayload::PostCreated { .. }
            | DomainEventPayload::CommentCreated { .. }
            | DomainEventPayload::PostHidden { .. }
            | DomainEventPayload::CommentEdited { .. }
    ) && actor_id.trim().is_empty()
    {
        return Err("blank_actor_id".into());
    }

    match payload {
        DomainEventPayload::PostCreated { visibility, .. }
        | DomainEventPayload::CommentCreated { visibility, .. }
        | DomainEventPayload::CommentEdited { visibility, .. } => {
            if visibility.trim().is_empty() {
                return Err("blank_visibility".into());
            }
        }
        DomainEventPayload::PostHidden { .. } => {}
        DomainEventPayload::SubscriptionChanged {
            resource_address,
            subscriber_delivery_address,
            ..
        } => {
            if resource_address.trim().is_empty() || subscriber_delivery_address.trim().is_empty() {
                return Err("blank_subscription_field".into());
            }
        }
    }

    if let DomainEventPayload::CommentEdited { body, .. } = payload
        && body.trim().is_empty()
    {
        return Err("blank_comment_body".into());
    }

    if let DomainEventPayload::PostCreated {
        body, body_format, ..
    } = payload
    {
        if body.trim().is_empty() {
            return Err("blank_post_body".into());
        }
        if !matches!(body_format.as_str(), "plain" | "markdown" | "html") {
            return Err("unknown_body_format".into());
        }
    }

    Ok(())
}

fn infer_event_type(payload: &DomainEventPayload) -> &'static str {
    match payload {
        DomainEventPayload::PostCreated { .. } => "post_created",
        DomainEventPayload::CommentCreated { .. } => "comment_created",
        DomainEventPayload::PostHidden { .. } => "post_hidden",
        DomainEventPayload::CommentEdited { .. } => "comment_edited",
        DomainEventPayload::SubscriptionChanged { .. } => "subscription_changed",
    }
}

fn infer_resource_id(payload: &DomainEventPayload) -> &str {
    match payload {
        DomainEventPayload::PostCreated { resource_id, .. }
        | DomainEventPayload::CommentCreated { resource_id, .. }
        | DomainEventPayload::PostHidden { resource_id, .. }
        | DomainEventPayload::CommentEdited { resource_id, .. } => resource_id,
        DomainEventPayload::SubscriptionChanged {
            resource_address, ..
        } => resource_address,
    }
}

fn infer_actor_id(payload: &DomainEventPayload) -> &str {
    match payload {
        DomainEventPayload::PostCreated { actor_id, .. }
        | DomainEventPayload::CommentCreated { actor_id, .. }
        | DomainEventPayload::PostHidden { actor_id, .. }
        | DomainEventPayload::CommentEdited { actor_id, .. } => actor_id,
        DomainEventPayload::SubscriptionChanged { .. } => "",
    }
}
