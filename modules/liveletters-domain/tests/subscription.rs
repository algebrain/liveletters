use liveletters_domain::{AccountId, ResourceAddress, Subscription};

#[test]
fn exposes_via_getters() {
    let resource = ResourceAddress::new("alice-publish@example.org").unwrap();
    let subscriber = AccountId::new("acct_bob").unwrap();
    let delivery = ResourceAddress::new("bob-feed@example.org").unwrap();
    let sub = Subscription::new(resource.clone(), subscriber.clone(), delivery.clone());
    assert_eq!(sub.resource_address(), &resource);
    assert_eq!(sub.subscriber_account_id(), &subscriber);
    assert_eq!(sub.subscriber_delivery_address(), &delivery);
}

#[test]
fn equality_holds_for_same_contents() {
    let a = Subscription::new(
        ResourceAddress::new("alice-publish@example.org").unwrap(),
        AccountId::new("acct_bob").unwrap(),
        ResourceAddress::new("bob-feed@example.org").unwrap(),
    );
    let b = Subscription::new(
        ResourceAddress::new("alice-publish@example.org").unwrap(),
        AccountId::new("acct_bob").unwrap(),
        ResourceAddress::new("bob-feed@example.org").unwrap(),
    );
    assert_eq!(a, b);
}
