# `liveletters-feed` — INTERFACE

## Назначение

`liveletters-feed` — библиотечный крейт, реализующий команду `lltt feed`. Печатает ленту текущего пользователя liveletters в человекочитаемом виде.

## Где находится интерфейс

- crate: `liveletters-feed`
- точка подключения: [`src/lib.rs`](src/lib.rs)
- алгоритм `run`: [`src/run.rs`](src/run.rs)
- форматирование вывода: [`src/print.rs`](src/print.rs)
- ISO 8601 UTC-форматирование Unix-времени: [`src/time.rs`](src/time.rs)

## Публичный API

```rust
pub use args::Args;
pub use error::FeedError;
pub use print::print_feed;
pub use run::run;
pub use liveletters_output::CommandContext;

pub const COMMAND_NAME: &str;
pub fn summary() -> &'static str;
pub fn crate_name() -> &'static str;

pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

`Args` собирается через `clap`-derive, см. ниже.

## `Args`

```rust
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Показать только N последних постов (по умолчанию все).
    #[arg(long)]
    pub limit: Option<usize>,
}
```

Единственный аргумент — `--limit <N>`. При отсутствии — печатается вся лента.

## `FeedError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка прикладного слоя: {0}")]
    AppCore(#[from] liveletters_app_core::AppCoreError),

    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),
}
```

## Алгоритм

1. `Store::open_for_home_dir(&ctx.home)` — открывает БД по `LIVELETTERS_HOME` / `<HOME>/.liveletters/`.
2. `liveletters_app_core::get_home_feed(&store, GetHomeFeedQuery)` — собирает `HomeFeed` (все посты, лежащие в БД; фильтрация по identity — отдельная задача).
3. `liveletters_config::load_identity(&ctx.home, &ctx.identity_name)` — `display_name` для шапки. Если конфиг отсутствует, в шапке используется `ctx.identity_name`.
4. `print_feed(&feed, &display, args.limit)` — печатает шапку, счётчик постов и сами посты.

## Что печатает

```
лента пользователя: Алиса
постов: 1 (показано: 1)

┌─ пост #post_1 от alice
│  visibility: public
│  2024-03-09 18:40:00 UTC
│  Привет, мир
└─
```

Если постов нет — печатает `(пусто)`. Если задан `--limit N` и постов больше N, в stdout попадают первые N постов; в строке счётчика — `(показано: N)`.

## Что НЕ печатает

- Пароли, токены, ключи шифрования.
- Содержимое писем (только метаданные + body поста).
- Цвет, индикаторы прогресса.

## Соседи

- [`liveletters-output`](../../modules/liveletters-output/INTERFACE.md) — `CommandContext`, `print_kv`.
- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — `Store::open_for_home_dir`, `list_posts`.
- [`liveletters-app-core`](../../modules/liveletters-app-core/INTERFACE.md) — `get_home_feed`, `HomeFeed`, `PostSummary`, `GetHomeFeedQuery`.
- [`liveletters-config`](../../modules/liveletters-config/INTERFACE.md) — `load_identity`, `IdentityConfig::display_name`.

## Тесты

- `tests/feed_print.rs` (4 теста) — печать `HomeFeed` с разным числом постов, скрытыми, с `limit`.
- `tests/common/mod.rs` — фикстуры `sample_post`, `feed_with`.
- Покрытие через бинарь: `apps/lltt/tests/cli_feed.rs` (4 теста) — `init`+`inbox import`+`feed` end-to-end.
