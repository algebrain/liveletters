//! Проверка, что `initialize_schema` создаёт таблицу `authors` с правильными
//! колонками и что все нужные таблицы имеют `FOREIGN KEY → authors(email)`.

mod common;

#[test]
fn authors_table_exists_with_required_columns() {
    let (store, _tmp) = common::open_temp_store();
    let cols = store.table_columns("authors").unwrap();
    assert!(!cols.is_empty(), "таблица `authors` должна существовать");

    let by_name: std::collections::HashMap<String, _> =
        cols.iter().map(|c| (c.name.clone(), c.clone())).collect();

    let pk = by_name.get("email").expect("колонка email обязательна");
    assert!(pk.pk, "email — PRIMARY KEY");
    assert_eq!(pk.column_type.to_uppercase(), "TEXT");
    assert!(!pk.nullable, "email NOT NULL");

    for must_have in ["nickname", "source", "first_seen_at", "updated_at"] {
        let col = by_name
            .get(must_have)
            .unwrap_or_else(|| panic!("колонка {must_have} обязательна в authors"));
        assert!(!col.nullable, "{must_have} NOT NULL");
    }
}

#[test]
fn all_fk_pointing_to_authors_are_enforced() {
    let (store, _tmp) = common::open_temp_store();

    // Минимальный набор таблиц, которые по плану должны ссылаться на authors.
    let tables = [
        "user_settings",
        "subscriptions",
        "local_subscriptions",
        "pending_subscriptions",
        "posts",
        "comments",
        "outbox",
        "bounce_records",
        "resources_owned",
    ];

    for table in tables {
        let cols = store
            .table_columns(table)
            .unwrap_or_else(|e| panic!("таблица {table} не найдена: {e}"));
        let email_col = cols
            .iter()
            .find(|c| c.name.contains("email"))
            .unwrap_or_else(|| panic!("таблица {table} должна иметь колонку *email*"));
        assert!(
            !email_col.nullable || table == "outbox" || table == "bounce_records",
            "таблица {table}: колонка {} должна быть NOT NULL",
            email_col.name
        );

        let fks = store
            .foreign_keys(table)
            .unwrap_or_else(|e| panic!("FK-лист для {table} не получен: {e}"));
        let fk_to_authors = fks.iter().find(|fk| fk.table == "authors");
        assert!(
            fk_to_authors.is_some(),
            "таблица {table} не имеет FOREIGN KEY → authors"
        );
    }
}

#[test]
fn inserting_post_with_unknown_author_fails() {
    use liveletters_store::PostRecord;

    let (store, _tmp) = common::open_temp_store();
    let res = store.save_post_record(&PostRecord {
        post_id: "post-1".into(),
        resource_email: "ghost@x.org".into(), // нет в authors
        author_email: "ghost@x.org".into(),
        created_at: 1,
        body: "тело".into(),
        visibility: "public".into(),
        hidden: false,
    });
    assert!(
        res.is_err(),
        "FK-ограничение не сработало: запись с призрачным author_email прошла"
    );
}

#[test]
fn display_names_table_does_not_exist() {
    let (store, _tmp) = common::open_temp_store();
    let names = store.list_table_names().unwrap();
    assert!(
        !names.iter().any(|n| n == "display_names"),
        "display_names должна быть удалена, но найдена в списке таблиц: {names:?}"
    );
}

#[test]
fn user_settings_has_author_email_and_no_nickname_or_email_address_columns() {
    let (store, _tmp) = common::open_temp_store();
    let cols = store.table_columns("user_settings").unwrap();
    let names: Vec<&str> = cols.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"author_email"),
        "user_settings.author_email обязателен, есть: {names:?}"
    );
    assert!(
        !names.contains(&"nickname"),
        "user_settings.nickname должен быть удалён (уехал в authors), но найден: {names:?}"
    );
    assert!(
        !names.contains(&"email_address"),
        "user_settings.email_address должен быть удалён (уехал в authors), но найден: {names:?}"
    );
}
