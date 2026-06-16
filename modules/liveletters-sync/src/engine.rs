use liveletters_i18n::{Vars, detect_system_locale, parse_locale, translate};
use liveletters_mail::{
    ReceivedEmail, decode_protocol_message, extract_liveletters_parts, parse_email,
};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage, encode_message};
use liveletters_store::{
    BounceRecord, CommentRecord, DeferredEventRecord, OutboxDelivery, OutboxRecord, PostRecord,
    RawEventRecord, RawMessageRecord, Store, StoreError, SubscriptionRecord,
};

use crate::{SyncError, SyncMessageOutcome, SyncReport};

pub struct SyncEngine<'a> {
    store: &'a Store,
    identity_filter: Option<IdentityFilter<'a>>,
    profile_id: Option<&'a str>,
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
            profile_id: None,
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
            profile_id: None,
        }
    }

    pub fn with_profile_id(mut self, profile_id: &'a str) -> Self {
        self.profile_id = Some(profile_id);
        self
    }

    pub fn ingest_batch(&self, messages: Vec<ReceivedEmail>) -> Result<SyncReport, SyncError> {
        let mut outcomes = Vec::new();

        for message in messages {
            let outcome = self.ingest_one(message)?;
            log_outcome(&outcome);
            outcomes.push(outcome);
        }

        Ok(SyncReport::new(outcomes))
    }

    pub fn reprocess_deferred(&self) -> Result<SyncReport, SyncError> {
        let deferred_records = self.store.list_deferred_event_records()?;
        let mut outcomes = Vec::new();

        for record in deferred_records {
            let payload: DomainEventPayload = serde_json::from_str(&record.payload_json)
                .map_err(SyncError::DeserializePayload)?;

            let envelope = liveletters_protocol::MessageEnvelope::new(
                "1",
                &record.event_type,
                infer_resource_id(&payload),
                &record.event_id,
            )
            .map_err(|e| SyncError::Invalid(format!("envelope: {e:?}")))?;

            let outcome = match self.apply_payload(&payload, infer_resource_id(&payload), &envelope)
            {
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
                // Письмо не парсится как protocol — возможно, это DSN-bounce.
                // Пробуем распознать bounce и обработать отдельно.
                if let Some(bounce) = liveletters_bounce::parse_dsn(&message.raw_message)
                    .ok()
                    .flatten()
                {
                    return self.handle_bounce(message, &bounce);
                }
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
            protocol_message.envelope(),
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

    /// Обработка DSN-bounce: сопоставляем с нашим исходящим по `Message-ID`,
    /// и если нашли `SubscriptionRequested` в outbox — удаляем pending.
    fn handle_bounce(
        &self,
        message: ReceivedEmail,
        bounce: &liveletters_bounce::BounceReport,
    ) -> Result<SyncMessageOutcome, SyncError> {
        let Some(message_id) = &bounce.original_message_id else {
            // Не наш DSN — игнор
            self.store.save_raw_message_record(&RawMessageRecord {
                message_id: message.message_id.clone(),
                raw_message: message.raw_message,
                status: "bounce.ignored".into(),
            })?;
            return Ok(SyncMessageOutcome::Filtered {
                message_id: message.message_id,
                event_id: "<unknown>".to_owned(),
                reason: "DSN без Original-Message-ID".to_owned(),
            });
        };

        let Some(outgoing) = self
            .store
            .find_outbox_by_message_id(message_id)
            .map_err(SyncError::Store)?
        else {
            // DSN для чужого Message-ID — игнор
            self.store.save_raw_message_record(&RawMessageRecord {
                message_id: message.message_id.clone(),
                raw_message: message.raw_message,
                status: "bounce.unmatched".into(),
            })?;
            return Ok(SyncMessageOutcome::Filtered {
                message_id: message.message_id,
                event_id: "<unknown>".to_owned(),
                reason: format!("DSN не соответствует нашему outbox: {message_id}"),
            });
        };

        if outgoing.event_type != "subscription_requested" {
            // Bounce для другого типа события (post/comment) — пока логируем,
            // полная обработка в будущем.
            liveletters_log::log_warn(format!(
                "bounce для {event_type} пока не обрабатывается: {message_id}",
                event_type = outgoing.event_type
            ));
            self.store.save_raw_message_record(&RawMessageRecord {
                message_id: message.message_id.clone(),
                raw_message: message.raw_message,
                status: "bounce.unsupported_event".into(),
            })?;
            return Ok(SyncMessageOutcome::Filtered {
                message_id: message.message_id,
                event_id: outgoing.event_id,
                reason: format!("bounce для {} пока не обрабатывается", outgoing.event_type),
            });
        }

        // Удаляем pending-подписку для текущего профиля. (B — единственный,
        // кто мог отправить этот `SubscriptionRequested` и получить bounce.)
        let profile_id = self.profile_id.unwrap_or("default");
        self.store
            .remove_pending_subscription(profile_id, &outgoing.resource_id)
            .map_err(SyncError::Store)?;
        // Сохраняем bounce_record
        self.store
            .save_bounce_record(&BounceRecord {
                original_message_id: message_id.clone(),
                event_id: Some(outgoing.event_id.clone()),
                final_recipient: Some(bounce.final_recipient.clone()),
                status_code: Some(bounce.status.clone()),
                diagnostic_code: Some(bounce.diagnostic_code.clone()),
                received_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            })
            .map_err(SyncError::Store)?;

        self.store.save_raw_message_record(&RawMessageRecord {
            message_id: message.message_id.clone(),
            raw_message: message.raw_message,
            status: "bounce.applied".into(),
        })?;

        liveletters_log::log_warn(format!(
            "доставка не удалась: subscription_requested -> {recipient}: {status} {diag}",
            recipient = bounce.final_recipient,
            status = bounce.status,
            diag = bounce.diagnostic_code,
        ));

        Ok(SyncMessageOutcome::Applied {
            message_id: message.message_id,
            event_id: outgoing.event_id,
        })
    }

    fn apply_payload(
        &self,
        payload: &DomainEventPayload,
        resource_id: &str,
        envelope: &liveletters_protocol::MessageEnvelope,
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
                body,
                body_format,
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

                let _ = body_format;

                self.store
                    .save_comment_record(&CommentRecord {
                        comment_id: comment_id.clone(),
                        post_id: post_id.clone(),
                        parent_comment_id: parent_comment_id.clone(),
                        author_id: actor_id.clone(),
                        created_at: *created_at,
                        body: body.clone(),
                        visibility: visibility.clone(),
                        hidden: false,
                    })
                    .map_err(ApplyEventError::Store)?;

                self.enqueue_redistribution(
                    resource_id,
                    envelope,
                    payload,
                    actor_id,
                    post_id,
                    body,
                )?;

                Ok(())
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
            DomainEventPayload::SubscriptionRequested {
                resource_address,
                subscriber_delivery_address,
                ..
            } => {
                // A — владелец ресурса — фиксирует подписку у себя,
                // чтобы знать подписчиков для последующих пересылок
                // (PostCreated/CommentCreated) и чтобы SubscriptionConfirmed
                // был идемпотентным.
                self.store
                    .save_subscription(&SubscriptionRecord {
                        resource_address: resource_address.clone(),
                        subscriber_delivery_address: subscriber_delivery_address.clone(),
                    })
                    .map_err(ApplyEventError::Store)?;

                // A автоматически отвечает B: формируем SubscriptionConfirmed
                // и кладём в свой outbox. B получит его через sync.
                let response = self.build_subscription_confirmed_response(
                    resource_address,
                    subscriber_delivery_address,
                )?;
                self.enqueue_outgoing(&response)?;
                Ok(())
            }
            DomainEventPayload::SubscriptionConfirmed {
                resource_address,
                subscriber_delivery_address,
                owner_nickname,
                owner_email,
                accepted,
                ..
            } => {
                if *accepted {
                    // pending → subscriptions + local + display_names
                    self.accept_pending_subscription(
                        resource_address,
                        subscriber_delivery_address,
                        owner_nickname,
                        owner_email,
                    )?;
                } else {
                    // A отклонил — удалить pending, ничего не создавать
                    self.decline_pending_subscription(
                        resource_address,
                        subscriber_delivery_address,
                    )?;
                }
                Ok(())
            }
            DomainEventPayload::SubscriptionRevoked {
                resource_address,
                subscriber_delivery_address,
                ..
            } => {
                let _ = self
                    .store
                    .delete_subscription(resource_address, subscriber_delivery_address)
                    .map_err(ApplyEventError::Store)?;
                Ok(())
            }
        }
    }

    /// A автоматически отвечает B: формирует `SubscriptionConfirmed` с профилем
    /// текущей идентичности (ник + email) и кладёт в свой outbox.
    /// Subject/body локализуются на языке отправителя (A) через
    /// `i18n_strings::subscription_confirmed_accepted`.
    fn build_subscription_confirmed_response(
        &self,
        resource_address: &str,
        subscriber_delivery_address: &str,
    ) -> Result<OutboxRecord, ApplyEventError> {
        let profile_id = self.profile_id.unwrap_or("default");
        let user = self
            .store
            .get_user_settings_record(profile_id)
            .map_err(ApplyEventError::Store)?
            .ok_or_else(|| {
                ApplyEventError::Invalid(format!(
                    "profile_id {profile_id} не найден в user_settings"
                ))
            })?;

        let owner_nickname = if user.nickname.is_empty() {
            // По решению пользователя: пустой профиль отправляем, логируем warning.
            liveletters_log::log_warn(format!(
                "user_settings.nickname пуст для {profile_id}; отправляем пустой профиль"
            ));
            String::new()
        } else {
            user.nickname.clone()
        };
        let owner_email = user.email_address.clone();
        let domain = owner_email
            .split_once('@')
            .map(|(_, d)| d.to_owned())
            .unwrap_or_else(|| "liveletters.invalid".to_owned());

        let event_id_str = format!(
            "subscription-confirmed:{}:{}:{}",
            resource_address,
            subscriber_delivery_address,
            owner_email.as_str()
        );
        let event_id = liveletters_domain::EventId::new(&event_id_str)
            .map_err(|e| ApplyEventError::Invalid(format!("event_id: {e:?}")))?;
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Локализованный subject/body на языке A.
        // (вызываем напрямую `liveletters_i18n`, чтобы не зависеть от
        // `liveletters-app-core` — граф зависимостей циклический.)
        let locale = parse_locale(&user.language).unwrap_or_else(|_| detect_system_locale());
        let subject = translate(
            "subscription_confirmed_accepted.subject",
            locale,
            Vars(&[("owner", &owner_nickname), ("resource", resource_address)]),
        )
        .expect("шаблон subscription_confirmed_accepted.subject присутствует в таблице");
        let body = translate(
            "subscription_confirmed_accepted.body",
            locale,
            Vars(&[("owner", &owner_nickname), ("resource", resource_address)]),
        )
        .expect("шаблон subscription_confirmed_accepted.body присутствует в таблице");

        let message = ProtocolMessage::new(
            MessageEnvelope::new(
                "1",
                "subscription_confirmed",
                resource_address,
                event_id.as_str(),
            )
            .map_err(|e| ApplyEventError::Invalid(format!("envelope: {e:?}")))?,
            &body,
            DomainEventPayload::SubscriptionConfirmed {
                resource_address: resource_address.to_owned(),
                subscriber_delivery_address: subscriber_delivery_address.to_owned(),
                owner_nickname,
                owner_email,
                accepted: true,
                created_at,
            },
        )
        .map_err(|e| ApplyEventError::Invalid(format!("protocol: {e:?}")))?;

        let message_body = encode_message(&message)
            .map_err(|e| ApplyEventError::Invalid(format!("encode: {e:?}")))?;

        let message_id = format!("<{}@{}>", event_id.as_str(), domain);

        Ok(OutboxRecord {
            event_id: event_id.as_str().to_owned(),
            event_type: "subscription_confirmed".to_owned(),
            resource_id: resource_address.to_owned(),
            delivery: OutboxDelivery::Direct(vec![subscriber_delivery_address.to_owned()]),
            message_body,
            message_id: Some(message_id),
            subject: Some(subject.clone()),
            // Локализованное тело живёт в отдельной колонке outbox,
            // а не в JSON. Поле `human_readable_body` в JSON пропущено
            // через `skip_serializing`, поэтому берём body здесь
            // (а не `message.human_readable_body()`, который
            // десериализуется в None).
            human_readable_body: Some(body),
        })
    }

    fn enqueue_outgoing(&self, record: &OutboxRecord) -> Result<(), ApplyEventError> {
        self.store
            .save_outbox_record(record)
            .map_err(ApplyEventError::Store)
    }

    /// B: `SubscriptionConfirmed { accepted: true }` →
    /// `pending → subscriptions + local_subscriptions + display_names`.
    /// Если pending нет (например, B уже отменил через `lltt sub cancel`),
    /// событие игнорируется с логом.
    fn accept_pending_subscription(
        &self,
        resource_address: &str,
        subscriber_delivery_address: &str,
        owner_nickname: &str,
        owner_email: &str,
    ) -> Result<(), ApplyEventError> {
        let profile_id = self.profile_id.unwrap_or("default");

        if self
            .store
            .find_pending_subscription(profile_id, resource_address)
            .map_err(ApplyEventError::Store)?
            .is_none()
        {
            liveletters_log::log_warn(format!(
                "SubscriptionConfirmed для {resource_address}, но pending нет; игнор (гонка состояний)"
            ));
            return Ok(());
        }

        self.store
            .save_subscription(&SubscriptionRecord {
                resource_address: resource_address.to_owned(),
                subscriber_delivery_address: subscriber_delivery_address.to_owned(),
            })
            .map_err(ApplyEventError::Store)?;
        self.store
            .add_local_subscription(profile_id, resource_address)
            .map_err(ApplyEventError::Store)?;
        self.store
            .save_display_name(owner_email, owner_nickname, "subscription_confirmed")
            .map_err(ApplyEventError::Store)?;
        self.store
            .remove_pending_subscription(profile_id, resource_address)
            .map_err(ApplyEventError::Store)?;
        Ok(())
    }

    /// B: `SubscriptionConfirmed { accepted: false }` → удалить pending.
    fn decline_pending_subscription(
        &self,
        resource_address: &str,
        subscriber_delivery_address: &str,
    ) -> Result<(), ApplyEventError> {
        let profile_id = self.profile_id.unwrap_or("default");
        self.store
            .remove_pending_subscription(profile_id, resource_address)
            .map_err(ApplyEventError::Store)?;
        liveletters_log::log_info(format!(
            "sub declined: {subscriber_delivery_address} rejected by {resource_address}"
        ));
        Ok(())
    }

    fn enqueue_redistribution(
        &self,
        resource_id: &str,
        envelope: &liveletters_protocol::MessageEnvelope,
        payload: &DomainEventPayload,
        author_id: &str,
        post_id: &str,
        body: &str,
    ) -> Result<(), ApplyEventError> {
        let subs = self
            .store
            .list_subscriptions_for_resource(resource_id)
            .map_err(ApplyEventError::Store)?;
        let recipients: Vec<String> = subs
            .into_iter()
            .map(|s| s.subscriber_delivery_address)
            .filter(|addr| addr != author_id)
            .collect();
        if recipients.is_empty() {
            return Ok(());
        }

        // Локализация на языке отправителя (владелец ресурса = profile_id движка).
        let profile_id = self.profile_id.unwrap_or("default");
        let locale = self
            .store
            .get_user_settings_record(profile_id)
            .ok()
            .flatten()
            .and_then(|r| parse_locale(&r.language).ok())
            .unwrap_or_else(detect_system_locale);

        let subject = translate(
            "comment_created_redistribute.subject",
            locale,
            Vars(&[("resource", resource_id)]),
        )
        .expect("шаблон comment_created_redistribute.subject присутствует в таблице");
        let human_body = translate(
            "comment_created_redistribute.body",
            locale,
            Vars(&[("sender", author_id), ("post_id", post_id), ("body", body)]),
        )
        .expect("шаблон comment_created_redistribute.body присутствует в таблице");

        let new_event_id = format!("redistribute:{}", envelope.event_id());
        let new_envelope = liveletters_protocol::MessageEnvelope::new(
            "1",
            envelope.event_type(),
            resource_id,
            &new_event_id,
        )
        .map_err(|e| ApplyEventError::Invalid(format!("envelope: {e:?}")))?;
        let message =
            liveletters_protocol::ProtocolMessage::new(new_envelope, &human_body, payload.clone())
                .map_err(|e| ApplyEventError::Invalid(format!("protocol: {e:?}")))?;
        let message_body = liveletters_protocol::encode_message(&message)
            .map_err(|e| ApplyEventError::Invalid(format!("encode: {e:?}")))?;

        let record = OutboxRecord {
            event_id: new_event_id,
            // event_type — технический идентификатор (Subject живёт в `subject`).
            event_type: envelope.event_type().to_owned(),
            resource_id: resource_id.to_owned(),
            delivery: OutboxDelivery::Direct(recipients),
            message_body,
            message_id: None,
            subject: Some(subject.clone()),
            // Тело письма хранится в отдельной колонке outbox;
            // в JSON оно не попадает (skip_serializing).
            human_readable_body: Some(human_body),
        };
        self.store
            .save_outbox_record(&record)
            .map_err(ApplyEventError::Store)
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
        DomainEventPayload::SubscriptionRequested {
            resource_address,
            subscriber_delivery_address,
            ..
        } => {
            if resource_address.trim().is_empty() || subscriber_delivery_address.trim().is_empty() {
                return Err("blank_subscription_field".into());
            }
        }
        DomainEventPayload::SubscriptionConfirmed {
            resource_address,
            subscriber_delivery_address,
            ..
        } => {
            if resource_address.trim().is_empty() || subscriber_delivery_address.trim().is_empty() {
                return Err("blank_subscription_field".into());
            }
        }
        DomainEventPayload::SubscriptionRevoked {
            resource_address,
            subscriber_delivery_address,
            ..
        } => {
            if resource_address.trim().is_empty() || subscriber_delivery_address.trim().is_empty() {
                return Err("blank_subscription_field".into());
            }
        }
    }

    match payload {
        DomainEventPayload::CommentCreated {
            body, body_format, ..
        } => {
            if body.trim().is_empty() {
                return Err("blank_comment_body".into());
            }
            if !matches!(body_format.as_str(), "plain" | "markdown" | "html") {
                return Err("unknown_body_format".into());
            }
        }
        DomainEventPayload::CommentEdited { body, .. } => {
            if body.trim().is_empty() {
                return Err("blank_comment_body".into());
            }
        }
        DomainEventPayload::PostCreated {
            body, body_format, ..
        } => {
            if body.trim().is_empty() {
                return Err("blank_post_body".into());
            }
            if !matches!(body_format.as_str(), "plain" | "markdown" | "html") {
                return Err("unknown_body_format".into());
            }
        }
        _ => {}
    }

    Ok(())
}

fn infer_event_type(payload: &DomainEventPayload) -> &'static str {
    match payload {
        DomainEventPayload::PostCreated { .. } => "post_created",
        DomainEventPayload::CommentCreated { .. } => "comment_created",
        DomainEventPayload::PostHidden { .. } => "post_hidden",
        DomainEventPayload::CommentEdited { .. } => "comment_edited",
        DomainEventPayload::SubscriptionRequested { .. } => "subscription_requested",
        DomainEventPayload::SubscriptionConfirmed { .. } => "subscription_confirmed",
        DomainEventPayload::SubscriptionRevoked { .. } => "subscription_revoked",
    }
}

fn infer_resource_id(payload: &DomainEventPayload) -> &str {
    match payload {
        DomainEventPayload::PostCreated { resource_id, .. }
        | DomainEventPayload::CommentCreated { resource_id, .. }
        | DomainEventPayload::PostHidden { resource_id, .. }
        | DomainEventPayload::CommentEdited { resource_id, .. } => resource_id,
        DomainEventPayload::SubscriptionRequested {
            resource_address, ..
        }
        | DomainEventPayload::SubscriptionConfirmed {
            resource_address, ..
        }
        | DomainEventPayload::SubscriptionRevoked {
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
        DomainEventPayload::SubscriptionRequested { .. }
        | DomainEventPayload::SubscriptionConfirmed { .. }
        | DomainEventPayload::SubscriptionRevoked { .. } => "",
    }
}

fn log_outcome(outcome: &SyncMessageOutcome) {
    let line = match outcome {
        SyncMessageOutcome::Applied {
            message_id,
            event_id,
        } => {
            format!("sync.ingest outcome=applied message_id={message_id} event_id={event_id}")
        }
        SyncMessageOutcome::Deferred {
            message_id,
            event_id,
            reason,
        } => format!(
            "sync.ingest outcome=deferred message_id={message_id} event_id={event_id} reason={reason}"
        ),
        SyncMessageOutcome::Replay {
            message_id,
            event_id,
            reason,
        } => format!(
            "sync.ingest outcome=replay message_id={message_id} event_id={event_id} reason={reason}"
        ),
        SyncMessageOutcome::Unauthorized {
            message_id,
            event_id,
            reason,
        } => format!(
            "sync.ingest outcome=unauthorized message_id={message_id} event_id={event_id} reason={reason}"
        ),
        SyncMessageOutcome::Invalid {
            message_id,
            event_id,
            reason,
        } => format!(
            "sync.ingest outcome=invalid message_id={message_id} event_id={event_id} reason={reason}"
        ),
        SyncMessageOutcome::Filtered {
            message_id,
            event_id,
            reason,
        } => format!(
            "sync.ingest outcome=filtered message_id={message_id} event_id={event_id} reason={reason}"
        ),
        SyncMessageOutcome::Malformed { message_id, reason } => {
            format!("sync.ingest outcome=malformed message_id={message_id} reason={reason}")
        }
        SyncMessageOutcome::Duplicate {
            message_id,
            event_id,
        } => {
            format!("sync.ingest outcome=duplicate message_id={message_id} event_id={event_id}")
        }
    };
    liveletters_log::log_info(line);
}
