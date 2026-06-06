use liveletters_posts::crate_name;

#[test]
fn crate_is_wired_into_workspace() {
    assert_eq!(crate_name(), "liveletters-posts");
}
