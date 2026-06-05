use liveletters_domain::{DomainError, ResourceAddress};

#[test]
fn rejects_blank_address() {
    let err = ResourceAddress::new("   ").unwrap_err();
    assert_eq!(err, DomainError::BlankIdentifier("resource_address"));
}

#[test]
fn rejects_address_without_at_sign() {
    let err = ResourceAddress::new("alice-publish-example.org").unwrap_err();
    assert_eq!(err, DomainError::InvalidAddress);
}

#[test]
fn round_trips_via_as_str() {
    let addr = ResourceAddress::new("alice-publish@example.org").unwrap();
    assert_eq!(addr.as_str(), "alice-publish@example.org");
}
