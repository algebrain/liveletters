# liveletters-status

## Назначение

`liveletters-status` — командный крейт `lltt status`. Печатает пять
бизнес-счётчиков домашнего каталога: число постов, число комментариев,
число отложенных событий, размер исходящей очереди, время последней
активности.

## Зона ответственности

Крейт отвечает за:

- открытие `Store` через `Store::open_for_home_dir(&ctx.home)`;
- вызов четырёх `count_*`-методов и `newest_post_created_at`;
- печать результата через `print_kv` (`liveletters-output`).

Крейт **не** отвечает за:

- отображение счётчиков синхронизации (это `liveletters-doctor`);
- показ отдельных записей outbox / raw_messages (это `liveletters-outbox`
  и `liveletters-inbox list`).

## Текущее состояние реализации

- `Args` — пустая структура (без clap-флагов);
- `StatusError` — единственный вариант `Store(StoreError)`;
- `StatusCounts` — публичный value-object с пятью полями;
- `print_status` — печатает 5 строк;
- `run` — открывает Store, вызывает счётчики, печатает;
- 4 интеграционных теста.

## Критерии готовности

- `cargo build -p liveletters-status` зелёный;
- `cargo test -p liveletters-status` зелёный;
- `lltt status` после `lltt init` печатает пять строк с нулями и
  строкой «нет активности».

## Связанные документы

- [`liveletters-store/src/posts.rs`](../../modules/liveletters-store/src/posts.rs) — `count_posts`, `newest_post_created_at`.
- [`liveletters-store/src/comments.rs`](../../modules/liveletters-store/src/comments.rs) — `count_comments`.
- [`liveletters-store/src/outbox.rs`](../../modules/liveletters-store/src/outbox.rs) — `count_outbox`.
- [`liveletters-store/src/raw.rs`](../../modules/liveletters-store/src/raw.rs) — `count_deferred_events`.
