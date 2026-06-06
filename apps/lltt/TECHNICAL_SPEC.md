# Бинарь `lltt` — TECHNICAL_SPEC

## Назначение

`lltt` — основной CLI проекта LiveLetters. Это тонкий диспетчер: он не содержит бизнес-логики, а только

1. разбирает аргументы командной строки через `clap`;
2. разрешает `CommandContext` (домашний каталог + имя текущего пользователя liveletters, если он нужен команде) из файловой системы;
3. проверяет, что домашний каталог существует и что задан текущий пользователь для команд, которым он обязателен;
4. вызывает `run(...)` нужного крейта из [`modules/`](../../modules).

Содержательная работа каждой команды вынесена в отдельный крейт с собственной парой `Args`/`Error` и собственными `INTERFACE.md`/`TECHNICAL_SPEC.md`.

## Архитектура

```
argv
  │
  ▼
Clap::parse()         ──►  Cli { command: Command }
                              │
                              ▼
                     context_mode_for(command)
                              │
                              ▼
                     build_context_for(mode)
                              │
                              ▼  читает LIVELETTERS_HOME
                              │  читает <home>/current-user только в RequiresCurrent
                     CommandContext { home, identity_name }
                              │
                              ▼
                 need_existing_home = mode != Init
                              │
                              ▼
                 if need_existing_home && !ctx.home.exists() → exit 2
                              │
                              ▼
                 match command { … }
                              │
                              ▼
                 <command>::run(&ctx, &args)
                              │
                              ▼
                 Ok → exit 0
                 Err → stderr "ошибка: …", exit 1
```

## Структура файлов

```
apps/lltt/
├── Cargo.toml
├── src/
│   ├── main.rs        # clap-дерево, dispatcher, обработка ошибок
│   └── context.rs     # resolve_home, resolve_current_user_name, build_context, ContextError
└── tests/
    ├── cli_smoke.rs           # --help показывает 14 подкоманд; неизвестная команда → ошибка; NoCurrentUser → код 2
    ├── cli_init.rs            # lltt init через бинарь
    ├── cli_cu.rs              # lltt cu, lltt cu posts через бинарь
    ├── cli_user.rs            # lltt user через бинарь
    ├── cli_sub.rs             # lltt sub (подписка/отписка/список) через бинарь
    ├── cli_feed.rs            # lltt feed через бинарь
    ├── cli_post.rs            # lltt post new через бинарь
    ├── cli_comment.rs         # lltt comment new через бинарь
    ├── cli_thread.rs          # lltt thread <post_id> через бинарь
    ├── cli_outbox.rs          # lltt outbox list через бинарь
    ├── cli_inbox_import.rs    # lltt inbox import <files>... через бинарь
    ├── cli_inbox_list.rs      # lltt inbox list (фильтр, лимит, no-init)
    ├── cli_inbox_show.rs      # lltt inbox show <id>
    ├── cli_status.rs          # lltt status через бинарь
    ├── cli_doctor.rs          # lltt doctor (Healthy + Degraded)
    ├── cli_doctor_verbose.rs  # lltt doctor --verbose (3 секции)
    ├── cli_settings.rs        # lltt settings show|set (включая обфускацию пароля)
    ├── cli_default.rs         # <HOME>/.liveletters/ без LIVELETTERS_HOME
    └── cli_sync_pull_push.rs  # lltt sync, sync pull, sync push с IMAP/SMTP-фикстурами на 127.0.0.1:0
```

`Cargo.toml` бинаря не имеет собственного `lib.rs`: это означает, что `apps/lltt` — только исполняемый файл, без библиотечной части.

## `src/main.rs` подробно

### `Cli` и `Command`

```rust
#[derive(Parser)]
#[command(name = "lltt", version, about = "LiveLetters CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init(init::Args),
    Cu(cu::Args),
    User(cu::Args),
    Feed(feed::Args),
    Inbox(inbox::Args),
    Post(post::Args),
    Comment(comment::Args),
    Outbox(outbox::Args),
    Thread(thread::Args),
    Status(status::Args),
    Doctor(doctor::Args),
    Settings(settings::Args),
    Sync(lltt_sync::Args),
}
```

Каждый вариант — это «тонкая обёртка» над `Args` соответствующего крейта. clap-derive сам разбирает аргументы подкоманды в `Args` крейта.

### `main`

```rust
fn main() -> ExitCode {
    let cli = Cli::parse();

    let mode = context_mode_for(&cli.command);
    let need_existing_home = !matches!(mode, context::ContextMode::Init);

    let ctx = match build_context_for(mode) {
        Ok(ctx) => ctx,
        Err(error) => {
            eprintln!("ошибка: {error}");
            return ExitCode::from(2);
        }
    };

    if need_existing_home && !ctx.home.exists() {
        eprintln!("ошибка: домашний каталог не существует: {}", ctx.home.display());
        eprintln!("запустите `lltt init` для инициализации");
        return ExitCode::from(2);
    }

    let result = match cli.command {
        Command::Init(args) => init::run(&ctx, &args),
        Command::Cu(args) => cu::run_current(&ctx, &args),
        Command::User(args) => cu::run_user(&ctx, &args),
        // … остальные варианты
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ошибка: {error}");
            ExitCode::from(1)
        }
    }
}
```

### `context_mode_for` и `build_context_for`

```rust
fn context_mode_for(command: &Command) -> context::ContextMode {
    match command {
        Command::Init(_) => context::ContextMode::Init,
        Command::User(_) => context::ContextMode::AllowMissingCurrent,
        Command::Cu(args) if cu_requires_current(&args.tokens) => {
            context::ContextMode::RequiresCurrent
        }
        Command::Cu(_) => context::ContextMode::AllowMissingCurrent,
        _ => context::ContextMode::RequiresCurrent,
    }
}

fn build_context_for(mode: context::ContextMode) -> Result<CommandContext, context::ContextError> {
    context::build_context(mode)
}
```

Режим `Init` не требует существующего домашнего каталога и не читает `<home>/current-user`. Режим `AllowMissingCurrent` требует домашний каталог, но не требует выбранного пользователя; он используется для `lltt user ...`, `lltt cu <имя>` и старых запрещённых форм `lltt cu list/add/rm/show <имя>`, чтобы пользователь получил подсказку от самой команды. Режим `RequiresCurrent` читает `<home>/current-user` и используется для команд, которые работают от имени текущей идентичности.

Имя текущего пользователя liveletters читается ТОЛЬКО из `<home>/current-user`; никаких переменных окружения (`LLTT_CU`) и никаких CLI-флагов (`--as`) сейчас не предусмотрено.

## `src/context.rs` подробно

```rust
use liveletters_store::resolve_data_dir_from_env;

pub fn resolve_home() -> PathBuf {
    resolve_data_dir_from_env().unwrap_or_else(|| PathBuf::from("."))
}

pub fn resolve_current_user_name(home: &Path) -> Result<String, ContextError> {
    match liveletters_config::read_current_identity(home) {
        Ok(name) => Ok(name),
        Err(ConfigError::NoCurrentUser(_)) => Err(ContextError::NoCurrentUser(home.join("current-user"))),
        Err(other) => Err(ContextError::Config(other)),
    }
}

pub enum ContextMode {
    Init,
    AllowMissingCurrent,
    RequiresCurrent,
}

pub fn build_context(mode: ContextMode) -> Result<CommandContext, ContextError> {
    let home = resolve_home();
    let identity_name = match mode {
        ContextMode::Init | ContextMode::AllowMissingCurrent => String::new(),
        ContextMode::RequiresCurrent => resolve_current_user_name(&home)?,
    };
    Ok(CommandContext { home, identity_name })
}
```

`resolve_data_dir_from_env` живёт в `liveletters-store` (как чистая функция, обёрнутая в `EnvOverrides::from_process()`) — это позволяет тестам `liveletters-store` проверять логику разрешения пути без мутации глобального окружения. `read_current_identity` живёт в `liveletters-config` (файловый ввод-вывод `<home>/current-user`).

## Контракт `CommandContext`

```rust
pub struct CommandContext {
    pub home: PathBuf,
    pub identity_name: String,
}
```

`identity_name` — это `String`; в коде используется как «имя текущей идентичности» (current identity), в публичной документации — как «имя текущего пользователя liveletters». Это одно и то же значение, просто в разной терминологии.

Контекст не содержит «пользователя операционной системы», не открывает БД и не делает сетевых вызовов. Это просто два значения, которые командный крейт использует для своих файловых операций. Подробнее — [`liveletters-output::CommandContext`](../../modules/liveletters-output/src/context.rs).

## Обработка ошибок

Каждый командный крейт возвращает `Result<(), Box<dyn Error + Send + Sync>>`. Бинарь `lltt` единообразно обрабатывает их:

- `Ok(())` → `ExitCode::SUCCESS` (0);
- `Err(e)` → печатает `ошибка: {e}` в `stderr` и возвращает `ExitCode::from(1)`.

Ошибки контекста (не удалось разрешить `LIVELETTERS_HOME`, домашний каталог не существует, файл `<home>/current-user` отсутствует для команды с режимом `RequiresCurrent`) обрабатываются отдельно, до диспетчеризации, и приводят к коду 2 — чтобы оболочка могла отличить «некорректное окружение» от «ошибки в самой команде».

### `ContextError`

```rust
pub enum ContextError {
    /// Файл `<home>/current-user` отсутствует для команды, которой нужен текущий пользователь.
    NoCurrentUser(PathBuf),
    /// Прочая ошибка конфигурации (TOML, IO и т.п.).
    Config(liveletters_config::ConfigError),
}
```

`NoCurrentUser` — частый случай сразу после `lltt init` или после ручного удаления `<home>/current-user`. Сообщение подсказывает последовательность: `lltt user init <имя>`, `lltt user add <имя> --from <файл>`, затем `lltt cu <имя>`.

## Зависимости

| Крейт | Зачем |
|---|---|
| `liveletters-config` | `read_current_identity` (чтение `<home>/current-user`), `ConfigError` |
| `liveletters-store` | `resolve_data_dir_from_env` (приоритет `LIVELETTERS_HOME` > `<HOME>` > `<USERPROFILE>` > `.`) |
| `liveletters-output` | `CommandContext` (общий тип для всех команд), `parse_visibility`, `read_body` |
| `liveletters-init` | команда `init` |
| `liveletters-cu` | команды `cu`, `cu posts` и `user` |
| `liveletters-feed` | команда `feed`, лента подписок |
| `liveletters-inbox` | команды `inbox import` и `inbox list` |
| `liveletters-post` | команда `post new` |
| `liveletters-comment` | команда `comment new` |
| `liveletters-outbox` | команда `outbox list` (read-only) |
| `liveletters-thread` | команда `thread <post_id>` |
| `liveletters-status`, `liveletters-doctor`, `liveletters-settings` | диагностические команды: `status`, `doctor`, `settings show|set` |
| `liveletters-lltt-sync` | команда `sync`: полный цикл `pull` затем `push`; отдельные команды `sync pull` и `sync push` |
| `clap` (derive) | разбор аргументов |

## Тесты

Все тесты — интеграционные, через `assert_cmd`:

- [`apps/lltt/tests/cli_smoke.rs`](tests/cli_smoke.rs):
  - `help_lists_all_fourteen_subcommands` — `lltt --help` содержит имена подкоманд (init, cu, user, sub, feed, inbox, post, comment, outbox, thread, status, doctor, settings, sync);
  - `unknown_subcommand_returns_error` — `lltt totally-bogus` возвращает ненулевой код и сообщение об ошибке;
  - `command_without_init_returns_no_current_user_error` — `lltt status` без `init` возвращает код 2 и сообщение про `<home>/current-user`;
  - `command_when_current_user_file_removed_returns_error` — `init`, создание `alice`, выбор `lltt cu alice`, затем удаление `<home>/current-user`, затем `lltt status` → код 2;
  - `status_succeeds_after_init` — после чистого `init` команда `lltt status` возвращает код 2, потому что текущий пользователь ещё не выбран.
- [`apps/lltt/tests/cli_init.rs`](tests/cli_init.rs) — 3 теста на `lltt init`.
- [`apps/lltt/tests/cli_cu.rs`](tests/cli_cu.rs) — тесты на `lltt cu`, `lltt cu posts` и запрет старых форм управления списком.
- [`apps/lltt/tests/cli_user.rs`](tests/cli_user.rs) — тесты на `lltt user init`, `lltt user add` и отсутствие автоматического выбора текущего пользователя.
- [`apps/lltt/tests/cli_sub.rs`](tests/cli_sub.rs) — тесты на `lltt sub` (подписка/отписка/список).
- [`apps/lltt/tests/cli_feed.rs`](tests/cli_feed.rs) — тесты на `lltt feed` как ленту подписок через бинарь.
- [`apps/lltt/tests/cli_post.rs`](tests/cli_post.rs) — тесты на `lltt post new` через бинарь.
- [`apps/lltt/tests/cli_comment.rs`](tests/cli_comment.rs) — тесты на `lltt comment new` через бинарь.
- [`apps/lltt/tests/cli_thread.rs`](tests/cli_thread.rs) — тесты на `lltt thread <post_id>` через бинарь.
- [`apps/lltt/tests/cli_outbox.rs`](tests/cli_outbox.rs) — тесты на `lltt outbox list` через бинарь.
- [`apps/lltt/tests/cli_inbox_import.rs`](tests/cli_inbox_import.rs) — тесты на `lltt inbox import <files>...` через бинарь.
- [`apps/lltt/tests/cli_inbox_list.rs`](tests/cli_inbox_list.rs) — 5 тестов на `lltt inbox list` (пусто, фильтр, лимит, невалидный статус, no-init).
- [`apps/lltt/tests/cli_inbox_show.rs`](tests/cli_inbox_show.rs) — 2 теста на `lltt inbox show <id>` (полное тело + неизвестный id).
- [`apps/lltt/tests/cli_status.rs`](tests/cli_status.rs) — тесты на `lltt status` через бинарь.
- [`apps/lltt/tests/cli_doctor.rs`](tests/cli_doctor.rs) — 2 теста на `lltt doctor` (Healthy + Degraded после malformed-импорта).
- [`apps/lltt/tests/cli_doctor_verbose.rs`](tests/cli_doctor_verbose.rs) — 2 теста на `lltt doctor --verbose` (3 секции; без `--verbose` вывод совпадает с легаси).
- [`apps/lltt/tests/cli_settings.rs`](tests/cli_settings.rs) — 3 теста на `lltt settings show|set` (включая обфускацию `smtp.password` через бинарь).
- [`apps/lltt/tests/cli_default.rs`](tests/cli_default.rs) — 2 теста на ветку `<HOME>/.liveletters/` без `LIVELETTERS_HOME`, с фейковой `HOME` в `/tmp`.
- [`apps/lltt/tests/cli_sync_pull_push.rs`](tests/cli_sync_pull_push.rs) — 4 e2e-теста на `lltt sync`, `lltt sync pull` и `lltt sync push` через бинарь с IMAP/SMTP-фикстурами на `127.0.0.1:0` (без mailpit/Docker): pull без настроек (понятная ошибка), pull с идемпотентным курсором (2 вызова → 1 + 0 писем), push с очисткой outbox и проверкой `RCPT TO` на SMTP-сервере, полный цикл `pull` затем `push`.

## Что осталось за пределами текущей версии

- В бинаре нет цветного вывода и индикаторов прогресса — весь вывод идёт через `println!`/`eprintln!` базовых крейтов.
- В бинаре нет встроенной справки по подкоманде (например, `lltt cu --help` пока спартанский вывод clap-derive).
- В бинаре нет `lltt --version` (на самом деле clap-derive добавляет `--version` из `Cargo.toml`; но семантическое версионирование пока не настроено).
