use liveletters_store::Store;

pub fn open_temp_store() -> (Store, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    (store, tmp)
}

pub fn save_user(store: &Store) {
    store
        .save_identity("default", "alice@example.test", "alice", None, "ru", true)
        .unwrap();
}
