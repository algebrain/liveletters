use liveletters_domain::crate_name;

#[test]
fn crate_is_available() {
    assert_eq!(crate_name(), "liveletters-domain");
}
