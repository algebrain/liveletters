use liveletters_utils::protocol_identity::{ProtocolIdentity, ProtocolIdentityError};

#[test]
fn parses_display_name_and_email() {
    let identity = ProtocolIdentity::parse("Alice <alice@example.com>").unwrap();

    assert_eq!(identity.nickname(), "Alice");
    assert_eq!(identity.email(), "alice@example.com");
}

#[test]
fn formats_canonical_wire_string() {
    let identity = ProtocolIdentity::new("Alice", "alice@example.com").unwrap();

    assert_eq!(identity.to_wire_string(), "Alice <alice@example.com>");
}

#[test]
fn trims_outer_parts_before_storing() {
    let identity = ProtocolIdentity::parse("  Alice  <  alice@example.com  >  ").unwrap();

    assert_eq!(identity.nickname(), "Alice");
    assert_eq!(identity.email(), "alice@example.com");
    assert_eq!(identity.to_wire_string(), "Alice <alice@example.com>");
}

#[test]
fn rejects_missing_angle_address() {
    let err = ProtocolIdentity::parse("alice@example.com").unwrap_err();

    assert!(matches!(
        err,
        ProtocolIdentityError::InvalidWireFormat { .. }
    ));
}

#[test]
fn rejects_blank_nickname() {
    let err = ProtocolIdentity::parse("<alice@example.com>").unwrap_err();

    assert!(matches!(err, ProtocolIdentityError::BlankNickname));
}

#[test]
fn rejects_blank_email() {
    let err = ProtocolIdentity::parse("Alice <>").unwrap_err();

    assert!(matches!(err, ProtocolIdentityError::BlankEmail));
}

#[test]
fn rejects_nested_or_extra_angle_brackets() {
    for raw in [
        "Alice <<alice@example.com>>",
        "Alice <alice@example.com> extra",
        "Alice <alice@example.com",
        "Alice alice@example.com>",
    ] {
        let err = ProtocolIdentity::parse(raw).unwrap_err();
        assert!(
            matches!(err, ProtocolIdentityError::InvalidWireFormat { .. }),
            "{raw:?} returned {err:?}"
        );
    }
}

#[test]
fn rejects_email_with_spaces() {
    let err = ProtocolIdentity::parse("Alice <alice @example.com>").unwrap_err();

    assert!(matches!(err, ProtocolIdentityError::InvalidEmail { .. }));
}

#[test]
fn serializes_to_json_string() {
    let identity = ProtocolIdentity::new("Alice", "alice@example.com").unwrap();

    assert_eq!(
        serde_json::to_string(&identity).unwrap(),
        "\"Alice <alice@example.com>\""
    );
}

#[test]
fn deserializes_from_json_string() {
    let identity: ProtocolIdentity = serde_json::from_str("\"Alice <alice@example.com>\"").unwrap();

    assert_eq!(identity.nickname(), "Alice");
    assert_eq!(identity.email(), "alice@example.com");
}
