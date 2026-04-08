use liveletters_store::crate_name;

#[test]
fn crate_is_wired_into_workspace() {
    assert_eq!(crate_name(), "liveletters-store");
}

