use liveletters_domain::{
    AccountId, EventId, ResourceAddress, SubscriptionAction, SubscriptionChanged, Timestamp,
};

#[test]
fn exposes_via_getters() {
    let event_id = EventId::new("sub:abc").unwrap();
    let resource = ResourceAddress::new("alice-publish@example.org").unwrap();
    let subscriber = AccountId::new("acct_bob").unwrap();
    let delivery = ResourceAddress::new("bob-feed@example.org").unwrap();
    let changed = SubscriptionChanged::new(
        event_id.clone(),
        resource.clone(),
        subscriber.clone(),
        delivery.clone(),
        SubscriptionAction::Subscribe,
        Timestamp::from_unix_seconds(1_700_000_000),
    );
    assert_eq!(changed.event_id(), &event_id);
    assert_eq!(changed.resource_address(), &resource);
    assert_eq!(changed.subscriber_account_id(), &subscriber);
    assert_eq!(changed.subscriber_delivery_address(), &delivery);
    assert_eq!(changed.action(), SubscriptionAction::Subscribe);
    assert_eq!(changed.created_at().as_unix_seconds(), 1_700_000_000);
}

#[test]
fn action_as_str_round_trip() {
    assert_eq!(SubscriptionAction::Subscribe.as_str(), "subscribe");
    assert_eq!(SubscriptionAction::Unsubscribe.as_str(), "unsubscribe");
}
