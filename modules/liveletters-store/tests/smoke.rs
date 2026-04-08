use liveletters_store::crate_name;

#[test]
fn crate_is_available() {
    assert_eq!(crate_name(), "liveletters-store");
}
