//! CRUD таблицы `authors`: save_author, get_author, list_authors, UPSERT-семантика.

mod common;

#[test]
fn save_author_then_get_returns_record() {
    let (store, _tmp) = common::open_temp_store();

    store
        .save_author("alice@x.org", "Алиса", "self")
        .expect("save_author не должен падать");

    let got = store
        .get_author("alice@x.org")
        .unwrap()
        .expect("запись есть");
    assert_eq!(got.email, "alice@x.org");
    assert_eq!(got.nickname, "Алиса");
    assert_eq!(got.source, "self");
    assert!(got.first_seen_at > 0);
    assert!(got.updated_at > 0);
}

#[test]
fn save_author_twice_preserves_first_seen_at_and_updates_nickname() {
    let (store, _tmp) = common::open_temp_store();

    store
        .save_author("bob@x.org", "Боб", "subscription_requested")
        .unwrap();
    let first = store.get_author("bob@x.org").unwrap().unwrap();
    let original_first_seen = first.first_seen_at;

    // Повторный save с другим ником (например, SubscriptionConfirmed обновил ник).
    store
        .save_author("bob@x.org", "Боб (подтверждён)", "subscription_confirmed")
        .unwrap();
    let second = store.get_author("bob@x.org").unwrap().unwrap();
    assert_eq!(second.nickname, "Боб (подтверждён)");
    assert_eq!(second.source, "subscription_confirmed");
    assert_eq!(
        second.first_seen_at, original_first_seen,
        "first_seen_at не должен меняться при UPSERT"
    );
    assert!(
        second.updated_at >= first.updated_at,
        "updated_at должен освежиться"
    );
}

#[test]
fn get_author_returns_none_for_unknown_email() {
    let (store, _tmp) = common::open_temp_store();
    let got = store.get_author("nobody@x.org").unwrap();
    assert!(
        got.is_none(),
        "для неизвестного email get_author должен вернуть None"
    );
}

#[test]
fn list_authors_returns_all_sorted_by_email() {
    let (store, _tmp) = common::open_temp_store();
    store.save_author("c@x.org", "Си", "self").unwrap();
    store.save_author("a@x.org", "А", "self").unwrap();
    store.save_author("b@x.org", "Б", "self").unwrap();

    let list = store.list_authors().unwrap();
    let emails: Vec<&str> = list.iter().map(|r| r.email.as_str()).collect();
    assert_eq!(emails, vec!["a@x.org", "b@x.org", "c@x.org"]);
}
