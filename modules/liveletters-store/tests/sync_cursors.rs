use liveletters_store::Store;
use tempfile::TempDir;

fn open_store() -> (TempDir, Store) {
    let tmp = TempDir::new().expect("tempdir");
    let store = Store::open_for_home_dir(tmp.path()).expect("store opens");
    (tmp, store)
}

#[test]
fn save_then_get() {
    let (_tmp, store) = open_store();
    store.save_sync_cursor("alice", 42).expect("save");
    let got = store.get_sync_cursor("alice").expect("get");
    assert_eq!(got, Some(42));
}

#[test]
fn get_returns_none_for_missing() {
    let (_tmp, store) = open_store();
    let got = store.get_sync_cursor("missing").expect("get missing");
    assert_eq!(got, None);
}

#[test]
fn save_overwrites() {
    let (_tmp, store) = open_store();
    store.save_sync_cursor("alice", 42).expect("save 42");
    store.save_sync_cursor("alice", 100).expect("save 100");
    let got = store.get_sync_cursor("alice").expect("get");
    assert_eq!(got, Some(100));
}
