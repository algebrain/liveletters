use liveletters_protocol::crate_name;

#[test]
fn crate_is_available() {
    assert_eq!(crate_name(), "liveletters-protocol");
}
