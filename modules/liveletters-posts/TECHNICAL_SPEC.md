# `liveletters-posts` — TECHNICAL_SPEC

## Назначение

Крейт содержит старую реализацию показа постов, переименованную из `liveletters-feed`. Он обслуживает `lltt cu posts`, а не `lltt feed`.

## Структура

```
modules/liveletters-posts/src/
├── args.rs    # Args { limit }
├── error.rs   # PostsError
├── lib.rs
├── print.rs   # print_posts
└── run.rs
```

## Поток выполнения

```rust
let identity = load_identity(&ctx.home, &ctx.identity_name)?;
let posts = get_current_user_posts(
    &store,
    GetCurrentUserPostsQuery {
        author_id: identity.account_id(),
    },
)?;
print_posts(&posts, identity.display_name(), args.limit);
```

`GetCurrentUserPostsQuery` фильтрует записи по `author_id`. Поэтому `cu posts` не показывает чужие посты, даже если они лежат в общей таблице `posts`.

## Тесты

- `modules/liveletters-posts/tests/posts_print.rs` — печать пустого списка, одного поста, ограничения `--limit`, скрытого поста.
- `apps/lltt/tests/cli_cu.rs` — `cu_posts_prints_current_users_posts_newest_first`.
- `modules/liveletters-store/tests/store_roundtrip.rs` — `list_posts_returns_newest_first`.

## Граница с `feed`

`feed` означает ленту подписок и реализуется отдельным крейтом `liveletters-feed`. В этом крейте не должно оставаться старых feed-имён из прежней реализации.
