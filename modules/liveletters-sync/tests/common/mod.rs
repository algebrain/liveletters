use liveletters_store::Store;
use tempfile::TempDir;

pub fn open_temp_store() -> (Store, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let store = Store::open_for_home_dir(tmp.path()).expect("store opens in temp home");
    (store, tmp)
}
