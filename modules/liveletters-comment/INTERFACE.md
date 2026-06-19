# `liveletters-comment` — INTERFACE

## Назначение

`liveletters-comment` — библиотечный крейт, реализующий команду `lltt comment`. Добавляет комментарий к записи в блоге: подбирает `comment_id` и `created_at`, читает тело из файла или stdin, записывает комментарий в `Store` и кладёт событие `comment_created` в `outbox`. Поддерживает вложенные ответы через `--parent`. Видимость комментария наследуется от исходной записи.

## Где находится интерфейс

- crate: `liveletters-comment`
- точка подключения: [`src/lib.rs`](src/lib.rs)
- разбор аргументов: [`src/args.rs`](src/args.rs)
- ошибки команды: [`src/error.rs`](src/error.rs)
- алгоритм `run`: [`src/run.rs`](src/run.rs)

## Публичный API

```rust
pub use args::{Args, CommentAction, NewArgs};
pub use error::CommentError;
pub use run::run;
pub use liveletters_output::CommandContext;

pub const COMMAND_NAME: &str;
pub fn summary() -> &'static str;
pub fn crate_name() -> &'static str;
pub fn print_created(comment_id: &str);
```

`run` имеет фиксированную сигнатуру:

```rust
pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

## `Args` / `CommentAction` / `NewArgs`

```rust
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: CommentAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum CommentAction {
    New(NewArgs),
}

#[derive(Debug, clap::Args)]
pub struct NewArgs {
    /// Идентификатор записи, к которой добавляется комментарий.
    #[arg(long)]
    pub post: String,

    /// Идентификатор родительского комментария (для вложенных ответов).
    #[arg(long)]
    pub parent: Option<String>,

    /// Файл с телом комментария. Если не указан — тело читается из stdin.
    #[arg(long)]
    pub body_file: Option<std::path::PathBuf>,

}
```

Поверхность CLI: `lltt comment new --post <id> [--parent <id>] [--body-file <path>]`.

## `CommentError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum CommentError {
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

    #[error("файл с телом комментария не найден: {path}")]
    BodyFileNotFound { path: std::path::PathBuf },

    #[error("тело комментария пустое")]
    EmptyBody,
}
```

## Алгоритм

1. `Store::open_for_home_dir(&ctx.home)` — открывает БД.
2. `liveletters_config::load_identity(...)` — `mail.publish`.
3. `liveletters_output::read_body(...)` — читает тело.
4. Если `body.trim().is_empty()` → `CommentError::EmptyBody`.
5. `AppCore::create_comment_from_identity(...)` — генерирует `comment_id`; подставляет `author_id = identity.publish`, `created_at = unix_millis_now() / 1000`, а видимость берёт из записи, к которой относится комментарий. Если пост с `args.post` не найден, `AppCore` возвращает `PostNotFound`.
6. `print_created(comment_id)`.

## Что печатает

```
комментарий создан: comment-1712345678901
```

## Что НЕ печатает

- Пароли, токены, ключи шифрования.
- Содержимое других комментариев.
- Содержимое `outbox`.

## Соседи

- [`liveletters-app-core`](../../modules/liveletters-app-core/INTERFACE.md) — `CreateCommentFromIdentityCommand`.
- [`liveletters-config`](../../modules/liveletters-config/INTERFACE.md) — `load_identity`, `MailSettings::publish`.
- [`liveletters-output`](../../modules/liveletters-output/INTERFACE.md) — `read_body`.
- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — `Store::open_for_home_dir`.

## Тесты

- `tests/flow.rs` (5 тестов):
  - `comment_new_creates_persisted_comment_with_default_visibility`
  - `comment_new_inherits_friends_only_visibility_from_post`
  - `comment_new_with_parent_creates_reply`
  - `comment_new_rejects_empty_body`
  - `comment_new_to_missing_post_errors`
- Покрытие через бинарь: `apps/lltt/tests/cli_comment.rs` — `init`+`post new`+`comment new`+SQL-проверка и отказ от устаревшего `--visibility`.
