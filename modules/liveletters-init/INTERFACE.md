# `liveletters-init` INTERFACE

## Назначение

`liveletters-init` — реализация команды `lltt init`. Подготавливает домашний каталог `lltt`: создаёт подкаталоги, корневую служебную БД совместимости и файл ключа обфускации паролей. Пользователя по умолчанию не создаёт. Команда идемпотентна: повторный запуск требует пустой каталог или флаг `--force`.

## Где находится интерфейс

- crate: `liveletters-init`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- `Args` — clap-аргументы (`force: bool`);
- `InitError` — типизированные ошибки инициализации;
- `run(&CommandContext, &Args) -> Result<(), Box<dyn Error + Send + Sync>>` — точка входа;
- `COMMAND_NAME: &str = "init"` — имя подкоманды в clap-дереве;
- `summary() -> &'static str` — короткое описание для `--help`;
- `crate_name() -> &'static str`.

Внутренние модули `args`, `error`, `run` не публикуются.

## `Args`

```rust
#[derive(clap::Args)]
pub struct Args {
    /// Перезаписать существующий каталог, если он не пуст.
    #[arg(long)]
    pub force: bool,
}
```

- `force == false` (по умолчанию): `init` отказывается работать, если `home` уже существует и не пуст (ошибка `InitError::AlreadyExists`).
- `force == true`: разрешает инициализацию непустого каталога без удаления существующих файлов.

Никаких других аргументов у команды нет: путь к `home` берётся из `CommandContext.home`, который в свою очередь получается из `LIVELETTERS_HOME` или `~/.liveletters/`.

## `InitError`

```rust
pub enum InitError {
    /// Каталог уже существует и не пуст; требуется --force.
    AlreadyExists(PathBuf),

    /// Ошибка открытия/инициализации SQLite-стора.
    Store(#[from] liveletters_store::StoreError),

    /// Ошибка чтения/записи файла ключа обфускации.
    Secret(#[from] liveletters_secret_box::SecretBoxError),

    /// Ошибка чтения/записи файла конфигурации.
    Config(#[from] liveletters_config::ConfigError),

    /// Прочая ошибка ввода-вывода.
    Io(#[from] std::io::Error),
}
```

Все варианты реализуют `std::error::Error` (через `thiserror`).

## `run`

```rust
pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn Error + Send + Sync>>
```

Шаги:

1. `ensure_home_empty(home, args.force)` — отказ, если каталог существует и не пуст без `--force`.
2. Создаёт `home` и подкаталоги `identities/`, `drafts/`, `inbox/`, `outbox-staged/`, `logs/`, `users/`.
3. Открывает БД через `Store::open_for_home_dir(home)` (создаёт `liveletters.sqlite3` и инициализирует схему).
4. Создаёт/открывает файл `mail-password-obfuscation.key` через `SecretBox::open_or_create`.
5. Печатает в stdout пять строк отчёта.

## Поведение и побочные эффекты

- команда **не модифицирует** `home`, если он не пуст и `--force` не передан;
- при `force == true` команда **создаёт** отсутствующие подкаталоги и служебные файлы, но не удаляет уже существующее содержимое;
- команда **не трогает** уже существующие идентичности в `identities/`;
- после успешного `init` нужно создать и выбрать пользователя через `lltt user init`, `lltt user add`, `lltt cu <имя>`; рабочая БД пользователя будет создана в `<home>/users/<имя>/`.

## Зависимости

- `liveletters-output` — `CommandContext`;
- `liveletters-secret-box` — `SecretBox::open_or_create`, `default_key_path`;
- `liveletters-store` — `Store::open_for_home_dir`;
- `clap` — derive `Args`;
- `thiserror` — derive `Error`.

## Пример использования (из теста)

```rust
use liveletters_init::{run, Args, CommandContext};
use std::path::PathBuf;

let tmp = tempfile::tempdir()?;
let ctx = CommandContext {
    home: tmp.path().to_path_buf(),
    state_home: tmp.path().to_path_buf(),
    identity_name: String::new(),
};
let args = Args { force: false };
run(&ctx, &args)?;

// Проверяем, что всё на месте
assert!(tmp.path().join("liveletters.sqlite3").exists());
assert!(tmp.path().join("mail-password-obfuscation.key").exists());
assert!(tmp.path().join("users").exists());
```
