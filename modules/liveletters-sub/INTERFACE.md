# Крейт `liveletters-sub` — INTERFACE

## Назначение

`liveletters-sub` — креЙт-команда для подкоманды `lltt sub`. Управляет подписками текущего пользователя liveletters на блоги (т.е. на `mail.publish` других пользователей).

Вся бизнес-логика сосредоточена здесь; `apps/lltt` лишь разбирает clap-аргументы и зовёт `run(&CommandContext, &Args)`.

## Где находится интерфейс

- бинарь: отсутствует (это library-креЙт);
- точка входа команды: [`src/run.rs`](src/run.rs);
- clap-аргументы: [`src/args.rs`](src/args.rs);
- типы ошибок: [`src/error.rs`](src/error.rs);
- реэкспорт: [`src/lib.rs`](src/lib.rs).

## Публичный API

```rust
pub use args::{Args, SubAction};
pub use error::SubError;
pub use liveletters_output::CommandContext;

pub const COMMAND_NAME: &str;
pub fn summary() -> &'static str;
pub fn crate_name() -> &'static str;
pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

`CommandContext` приходит из `liveletters-output` и содержит `home: PathBuf` + `identity_name: String`.

## `Args`

```rust
pub struct Args {
    pub tokens: Vec<String>,
}
```

Кладётся прямо в кортеж clap-варианта `Command::Sub(liveletters_sub::Args)` в `apps/lltt/src/main.rs`. Содержит **все позиционные токены после `lltt sub`**, без какой-либо clap-магии на стороне этого креЙта (формат `trailing_var_arg`).

Первый токен выбирает операцию (`SubAction`), остальные — аргументы операции.

## `SubAction`

```rust
pub enum SubAction {
    Subscribe { resource_address: String },
    List,
    Rm        { resource_address: String },
}
```

| Первый токен     | Под-аргументы       | Получаемый `SubAction`           |
|------------------|---------------------|----------------------------------|
| `list`           | —                   | `List`                           |
| `rm`             | `<адрес>`           | `Rm { resource_address }`        |
| (любой `<адрес>`)| —                   | `Subscribe { resource_address }` |

Имена `list` и `rm` зарезервированы (без учёта регистра). Если первый токен не распознан и не похож на адрес — `SubError::InvalidArgs` с человекочитаемым сообщением.

## `SubError`

```rust
pub enum SubError {
    Config(#[from] liveletters_config::ConfigError),
    Io(#[from] std::io::Error),
    Domain(#[from] liveletters_domain::DomainError),
    Store(#[from] liveletters_store::StoreError),
    AppCore(#[from] liveletters_app_core::AppCoreError),
    InvalidArgs(String),
}
```

`Display` реализован через `thiserror`. `From<…>` реализованы для типов, указанных в вариантах; обёртка до `Box<dyn Error + Send + Sync>` происходит в `lib.rs::run`. Вариант `InvalidArgs` несёт человекочитаемое сообщение, формируемое парсером токенов (`parse_action`).

## Побочные эффекты

| Операция | Файлы / таблицы |
|---|---|
| `Subscribe` | 1) апдейт `<home>/identities/<текущий>.toml` (добавление адреса в `meta.subscriptions`); 2) операции `liveletters_app_core::AppCore::subscribe` над таблицей `subscriptions` (запись `INSERT`). |
| `List`     | Только чтение (`identities/<текущий>.toml` + `subscriptions` + `subscriptions` по `mail.publish`). |
| `Rm`       | 1) апдейт `<home>/identities/<текущий>.toml` (удаление адреса из `meta.subscriptions`); 2) операция `liveletters_app_core::AppCore::unsubscribe` (удаление из `subscriptions`). |

## Вывод

Операции печатают человекочитаемые строки на `stdout`:

- `Subscribe`: `подписан на <resource>: посты будут приходить на <delivery>`
- `List`:     две секции — «подписан на:» (по одной строке на адрес или `(пусто)`) и «мои подписчики:» (по строке `<account_id>  →  <delivery>` или `(пусто)`).
- `Rm`:       `отписан от <resource>`

Ошибки печатает бинарь `apps/lltt` (он ловит `Err` и печатает `ошибка: <текст>` в `stderr`).

## Соседи

- [`liveletters-output`](../../modules/liveletters-output/INTERFACE.md) — `CommandContext`.
- [`liveletters-config`](../../modules/liveletters-config/INTERFACE.md) — `IdentityConfig`, `IdentityMeta`, `save_identity`, `load_identity`.
- [`liveletters-domain`](../../modules/liveletters-domain/INTERFACE.md) — `ResourceAddress::new`, `DomainError`.
- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — `Store::open_for_home_dir`.
- [`liveletters-app-core`](../../modules/liveletters-app-core/INTERFACE.md) — `AppCore::subscribe`/`unsubscribe`/`list_subscriptions`, `SubscribeCommand`/`UnsubscribeCommand`/`ListSubscriptionsQuery`.

## Тесты

- `tests/flow.rs` — интеграционные тесты: подписка, отписка, список, невалидный адрес, пустые токены.
- Покрытие через бинарь: `apps/lltt/tests/cli_sub.rs` — тесты на `lltt sub` (подписка/отписка/список) через бинарь.
