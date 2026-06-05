# `liveletters-thread` — INTERFACE

## Назначение

`liveletters-thread` — библиотечный крейт, реализующий команду `lltt thread`. Печатает обсуждение (запись + дерево комментариев) в человекочитаемом виде. Команда read-only.

## Где находится интерфейс

- crate: `liveletters-thread`
- точка подключения: [`src/lib.rs`](src/lib.rs)
- разбор аргументов: [`src/args.rs`](src/args.rs)
- ошибки команды: [`src/error.rs`](src/error.rs)
- алгоритм `run` + печать: [`src/run.rs`](src/run.rs)

## Публичный API

```rust
pub use args::Args;
pub use error::ThreadError;
pub use run::{print_thread, run};
pub use liveletters_output::CommandContext;

pub const COMMAND_NAME: &str;
pub fn summary() -> &'static str;
pub fn crate_name() -> &'static str;
```

`run` имеет фиксированную сигнатуру:

```rust
pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

## `Args`

```rust
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Идентификатор записи, для которой нужно показать обсуждение.
    pub post_id: String,
}
```

Поверхность CLI: `lltt thread <post_id>`.

## `ThreadError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ThreadError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка прикладного слоя: {0}")]
    AppCore(#[from] liveletters_app_core::AppCoreError),
}
```

## Алгоритм

1. `Store::open_for_home_dir(&ctx.home)` — открывает БД.
2. `liveletters_app_core::get_post_thread(&store, GetPostThreadQuery { post_id })` — возвращает `PostThread { post: PostSummary, comments: Vec<CommentSummary> }`. Если пост не найден, `AppCore` возвращает `PostNotFound`.
3. `print_thread(&thread)` — печатает запись, счётчик комментариев и дерево.

## Что печатает

```
┌─ пост #post_1 от alice
│  visibility: public
│  Запись для thread
└─

комментарии: 2

  • bob (comment-1)
        Корневой
    ↳ alice (comment-2)
        Ответ
```

Если комментариев нет:

```
комментарии: 0

(нет комментариев)
```

## Что НЕ печатает

- Пароли, токены, ключи шифрования.
- Скрытые комментарии помечаются `(скрыт)`, но их тело не скрывается.
- Полные тексты писем (если комментарий пришёл из `inbox`).

## Соседи

- [`liveletters-app-core`](../../modules/liveletters-app-core/INTERFACE.md) — `get_post_thread`, `GetPostThreadQuery`, `PostThread`, `PostSummary`, `CommentSummary`.
- [`liveletters-output`](../../modules/liveletters-output/INTERFACE.md) — `print_kv`.
- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — `Store::open_for_home_dir`.

## Тесты

- `src/run.rs::tests` (2 unit-теста) — `render_tree_shows_root_and_reply_with_prefix`, `render_tree_with_no_comments_is_empty`.
- `tests/flow.rs` (3 integration-теста):
  - `thread_for_existing_post_prints_post_and_no_comments_marker`
  - `thread_for_post_with_root_and_reply_prints_tree`
  - `thread_for_missing_post_errors`
- Покрытие через бинарь: `apps/lltt/tests/cli_thread.rs` (2 теста) — `init`+`post new`+`thread`+проверка stdout.
