# `liveletters-post` — INTERFACE

## Назначение

`liveletters-post` — библиотечный крейт, реализующий команду `lltt post`. Создаёт новую запись в блоге текущего пользователя liveletters: подбирает `post_id` и `created_at`, читает тело из файла или stdin, записывает запись в `Store` и кладёт событие `post_created` в `outbox`.

## Где находится интерфейс

- crate: `liveletters-post`
- точка подключения: [`src/lib.rs`](src/lib.rs)
- разбор аргументов: [`src/args.rs`](src/args.rs)
- ошибки команды: [`src/error.rs`](src/error.rs)
- алгоритм `run`: [`src/run.rs`](src/run.rs)

## Публичный API

```rust
pub use args::{Args, NewArgs, PostAction};
pub use error::PostError;
pub use run::run;
pub use liveletters_output::CommandContext;

pub const COMMAND_NAME: &str;
pub fn summary() -> &'static str;
pub fn crate_name() -> &'static str;
pub fn print_created(post_id: &str);
```

`run` имеет фиксированную сигнатуру:

```rust
pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

## `Args` / `PostAction` / `NewArgs`

```rust
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: PostAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum PostAction {
    New(NewArgs),
}

#[derive(Debug, clap::Args)]
pub struct NewArgs {
    /// Файл с телом записи. Если не указан — тело читается из stdin.
    #[arg(long)]
    pub body_file: Option<std::path::PathBuf>,

    /// Уровень видимости: `public` или `friends_only`.
    #[arg(long, default_value = "public")]
    pub visibility: String,
}
```

Поверхность CLI: `lltt post new [--body-file <path>] [--visibility <level>]`.

## `PostError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum PostError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка прикладного слоя: {0}")]
    AppCore(#[from] liveletters_app_core::AppCoreError),

    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),

    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    IoFromOutput(String),

    #[error("файл с телом записи не найден: {path}")]
    BodyFileNotFound { path: std::path::PathBuf },

    #[error("{0}")]
    UnknownVisibility(String),

    #[error("тело записи пустое")]
    EmptyBody,
}
```

## Алгоритм

1. `Store::open_for_home_dir(&ctx.home)` — открывает БД.
2. `liveletters_config::load_identity(&ctx.home, &ctx.identity_name)` — достаёт `account_id` и `mail.publish`.
3. `liveletters_output::read_body(args.body_file, &mut stdin)` — читает тело (файл или stdin). Если файл указан, но не существует, возвращается `BodyFileNotFound`.
4. Если `body.trim().is_empty()` → `PostError::EmptyBody`.
5. `liveletters_output::parse_visibility(&args.visibility)` — принимает только `public` и `friends_only`. Иные значения → `UnknownVisibility`.
6. `AppCore::create_post_from_identity(...)` — генерирует `post_id`, подставляет `resource_id = identity.publish`, `author_id = identity.account_id`, `created_at = unix_millis_now() / 1000`.
7. `print_created(post_id)` — печатает `запись создана: <post_id>`.

## Что печатает

```
запись создана: post-1712345678901
```

## Что НЕ печатает

- Пароли, токены, ключи шифрования.
- Содержимое других записей или комментариев.
- Содержимое `outbox` (для этого есть команда `lltt outbox`).

## Соседи

- [`liveletters-app-core`](../../modules/liveletters-app-core/INTERFACE.md) — `CreatePostFromIdentityCommand`, `Identity`, `Visibility`.
- [`liveletters-config`](../../modules/liveletters-config/INTERFACE.md) — `load_identity`, `MailSettings::publish`.
- [`liveletters-output`](../../modules/liveletters-output/INTERFACE.md) — `read_body`, `parse_visibility`.
- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — `Store::open_for_home_dir`.

## Тесты

- `src/run.rs::tests` (1 тест) — `identity_from_config_uses_publish_and_account_id`.
- `tests/flow.rs` (4 теста):
  - `post_new_creates_persisted_post_with_default_visibility`
  - `post_new_with_friends_only_visibility`
  - `post_new_rejects_empty_body`
  - `post_new_rejects_unknown_visibility`
- Покрытие через бинарь: `apps/lltt/tests/cli_post.rs` (2 теста) — `init`+`post new`+SQL-проверка.
