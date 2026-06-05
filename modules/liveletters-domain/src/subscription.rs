use crate::{AccountId, ResourceAddress};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subscription {
    resource_address: ResourceAddress,
    subscriber_account_id: AccountId,
    subscriber_delivery_address: ResourceAddress,
}

impl Subscription {
    pub fn new(
        resource_address: ResourceAddress,
        subscriber_account_id: AccountId,
        subscriber_delivery_address: ResourceAddress,
    ) -> Self {
        Self {
            resource_address,
            subscriber_account_id,
            subscriber_delivery_address,
        }
    }

    pub fn resource_address(&self) -> &ResourceAddress {
        &self.resource_address
    }

    pub fn subscriber_account_id(&self) -> &AccountId {
        &self.subscriber_account_id
    }

    pub fn subscriber_delivery_address(&self) -> &ResourceAddress {
        &self.subscriber_delivery_address
    }
}
