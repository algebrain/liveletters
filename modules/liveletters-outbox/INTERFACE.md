# `liveletters-outbox` — INTERFACE

## Назначение

`liveletters-outbox` — библиотечный крейт, реализующий команду `lltt outbox`. Печатает список неотправленных событий (`OutboxRecord`) из локального `Store` в человекочитаемом виде. Команда read-only: никаких сетевых вызовов, никаких изменений в БД.

## Где находится интерфейс

- crate: `liveletters-outbox`
- точка подключения: [`src/lib.rs`](src/lib.rs)
- разбор аргументов: [`src/args.rs`](src/args.rs)
- ошибки команды: [`src/error.rs`](src/error.rs)
- алгоритм `run` + печать: [`src/run.rs`](src/run.rs)

## Публичный API

```rust
pub use args::{Args, OutboxAction};
pub use error::OutboxError;
pub use run::{print_summary, run};
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

## `Args` / `OutboxAction`

```rust
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: OutboxAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum OutboxAction {
    /// Показать неотправленные события (read-only).
    List,
}
```

Поверхность CLI: `lltt outbox list`.

## `OutboxError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка прикладного слоя: {0}")]
    AppCore(#[from] liveletters_app_core::AppCoreError),
}
```

## Алгоритм

1. `Store::open_for_home_dir(&ctx.home)` — открывает БД.
2. `liveletters_app_core::get_pending_outbox(&store, GetPendingOutboxQuery)` — возвращает `PendingOutbox { entries: Vec<OutboxEntry> }`.
3. `print_summary(&pending)` — печатает заголовок и таблицу.

## Что печатает

```
неотправленные события: 2

event_id                  event_type         resource_id
post-created:post-1       post_created       alice-publish@example.org
comment-created:comment-1 comment_created   alice-publish@example.org
```

Если событий нет:

```
неотправленные события: 0

(пусто)
```

## Что НЕ печатает

- Пароли, токены, ключи шифрования.
- `message_body` события (только метаданные). Полное тело письма остаётся в БД и используется синком.
- Содержимое других таблиц.

## Соседи

- [`liveletters-app-core`](../../modules/liveletters-app-core/INTERFACE.md) — `get_pending_outbox`, `GetPendingOutboxQuery`, `OutboxEntry`, `PendingOutbox`.
- [`liveletters-output`](../../modules/liveletters-output/INTERFACE.md) — `print_kv`, `print_table`.
- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — `Store::open_for_home_dir`, `OutboxRecord`.

## Тесты

- `tests/flow.rs` (3 теста):
  - `outbox_list_empty_store_succeeds`
  - `outbox_list_shows_pending_post_created`
  - `print_summary_works_with_empty_and_populated`
- Покрытие через бинарь: `apps/lltt/tests/cli_outbox.rs` (2 теста) — `init`+`outbox list`+проверка stdout.
