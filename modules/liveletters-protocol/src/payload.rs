use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEventPayload {
    PostCreated {
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        body: String,
        body_format: String,
        visibility: String,
    },
    CommentCreated {
        comment_id: String,
        post_id: String,
        parent_comment_id: Option<String>,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        body: String,
        body_format: String,
        visibility: String,
    },
    PostHidden {
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
    },
    CommentEdited {
        comment_id: String,
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        body: String,
        visibility: String,
    },
    SubscriptionChanged {
        resource_address: String,
        subscriber_delivery_address: String,
        active: bool,
        created_at: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WireDomainEventPayload {
    PostCreated {
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        #[serde(default)]
        body: String,
        #[serde(default = "default_body_format")]
        body_format: String,
        visibility: String,
    },
    CommentCreated {
        comment_id: String,
        post_id: String,
        parent_comment_id: Option<String>,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        #[serde(default)]
        body: String,
        #[serde(default = "default_body_format")]
        body_format: String,
        visibility: String,
    },
    PostHidden {
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
    },
    CommentEdited {
        comment_id: String,
        post_id: String,
        resource_id: String,
        actor_id: String,
        created_at: u64,
        body: String,
        visibility: String,
    },
    SubscriptionChanged {
        resource_address: String,
        subscriber_delivery_address: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        active: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subscriber_account_id: Option<String>,
        created_at: u64,
    },
}

impl Serialize for DomainEventPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        WireDomainEventPayload::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DomainEventPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WireDomainEventPayload::deserialize(deserializer)?;
        DomainEventPayload::try_from(wire).map_err(de::Error::custom)
    }
}

impl From<&DomainEventPayload> for WireDomainEventPayload {
    fn from(value: &DomainEventPayload) -> Self {
        match value {
            DomainEventPayload::PostCreated {
                post_id,
                resource_id,
                actor_id,
                created_at,
                body,
                body_format,
                visibility,
            } => Self::PostCreated {
                post_id: post_id.clone(),
                resource_id: resource_id.clone(),
                actor_id: actor_id.clone(),
                created_at: *created_at,
                body: body.clone(),
                body_format: body_format.clone(),
                visibility: visibility.clone(),
            },
            DomainEventPayload::CommentCreated {
                comment_id,
                post_id,
                parent_comment_id,
                resource_id,
                actor_id,
                created_at,
                body,
                body_format,
                visibility,
            } => Self::CommentCreated {
                comment_id: comment_id.clone(),
                post_id: post_id.clone(),
                parent_comment_id: parent_comment_id.clone(),
                resource_id: resource_id.clone(),
                actor_id: actor_id.clone(),
                created_at: *created_at,
                body: body.clone(),
                body_format: body_format.clone(),
                visibility: visibility.clone(),
            },
            DomainEventPayload::PostHidden {
                post_id,
                resource_id,
                actor_id,
                created_at,
            } => Self::PostHidden {
                post_id: post_id.clone(),
                resource_id: resource_id.clone(),
                actor_id: actor_id.clone(),
                created_at: *created_at,
            },
            DomainEventPayload::CommentEdited {
                comment_id,
                post_id,
                resource_id,
                actor_id,
                created_at,
                body,
                visibility,
            } => Self::CommentEdited {
                comment_id: comment_id.clone(),
                post_id: post_id.clone(),
                resource_id: resource_id.clone(),
                actor_id: actor_id.clone(),
                created_at: *created_at,
                body: body.clone(),
                visibility: visibility.clone(),
            },
            DomainEventPayload::SubscriptionChanged {
                resource_address,
                subscriber_delivery_address,
                active,
                created_at,
            } => Self::SubscriptionChanged {
                resource_address: resource_address.clone(),
                subscriber_delivery_address: subscriber_delivery_address.clone(),
                active: Some(*active),
                action: None,
                subscriber_account_id: None,
                created_at: *created_at,
            },
        }
    }
}

impl TryFrom<WireDomainEventPayload> for DomainEventPayload {
    type Error = String;

    fn try_from(value: WireDomainEventPayload) -> Result<Self, Self::Error> {
        match value {
            WireDomainEventPayload::PostCreated {
                post_id,
                resource_id,
                actor_id,
                created_at,
                body,
                body_format,
                visibility,
            } => Ok(Self::PostCreated {
                post_id,
                resource_id,
                actor_id,
                created_at,
                body,
                body_format,
                visibility,
            }),
            WireDomainEventPayload::CommentCreated {
                comment_id,
                post_id,
                parent_comment_id,
                resource_id,
                actor_id,
                created_at,
                body,
                body_format,
                visibility,
            } => Ok(Self::CommentCreated {
                comment_id,
                post_id,
                parent_comment_id,
                resource_id,
                actor_id,
                created_at,
                body,
                body_format,
                visibility,
            }),
            WireDomainEventPayload::PostHidden {
                post_id,
                resource_id,
                actor_id,
                created_at,
            } => Ok(Self::PostHidden {
                post_id,
                resource_id,
                actor_id,
                created_at,
            }),
            WireDomainEventPayload::CommentEdited {
                comment_id,
                post_id,
                resource_id,
                actor_id,
                created_at,
                body,
                visibility,
            } => Ok(Self::CommentEdited {
                comment_id,
                post_id,
                resource_id,
                actor_id,
                created_at,
                body,
                visibility,
            }),
            WireDomainEventPayload::SubscriptionChanged {
                resource_address,
                subscriber_delivery_address,
                active,
                action,
                created_at,
                ..
            } => {
                let active = match (active, action.as_deref()) {
                    (Some(active), _) => active,
                    (None, Some("subscribe")) | (None, None) => true,
                    (None, Some("unsubscribe")) => false,
                    (None, Some(other)) => {
                        return Err(format!("unknown subscription action: {other}"));
                    }
                };

                Ok(Self::SubscriptionChanged {
                    resource_address,
                    subscriber_delivery_address,
                    active,
                    created_at,
                })
            }
        }
    }
}

fn default_body_format() -> String {
    "plain".to_owned()
}
