use liveletters_store::Store;

/// Создаёт `Store` поверх временной домашней директории.
/// Возвращаемая `TempDir` обязана жить в одной области видимости
/// с `Store` — иначе каталог будет удалён раньше, чем `Store`
/// закроет файл.
pub fn open_temp_store() -> (Store, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    (store, tmp)
}
