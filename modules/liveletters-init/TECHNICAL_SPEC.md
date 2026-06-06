# `liveletters-init` TECHNICAL_SPEC

## Назначение

`liveletters-init` — единственная команда, которая **создаёт** структуру домашнего каталога `lltt` на диске. Все остальные команды (`cu`, `sub`, `feed`, `inbox`, `doctor`, …) предполагают, что каталог уже инициализирован, и при его отсутствии возвращают ошибку.

## Зона ответственности

- идемпотентное создание каталога `home` и его подкаталогов;
- инициализация корневой служебной SQLite-БД совместимости через `liveletters-store::Store::open_for_home_dir`;
- создание/открытие файла `mail-password-obfuscation.key` через `liveletters-secret-box`;

## Что команда не делает

- не генерирует пароли и не подключается к почтовым серверам;
- не создаёт ничего вне `ctx.home`;
- не модифицирует переменные окружения (это задача команды `home`);
- не печатает пароли и не логирует секреты;
- не создаёт пользователя по умолчанию и не выбирает текущего пользователя.

## Алгоритм

```
fn run_inner(home, force):
    ensure_home_empty(home, force)
    fs::create_dir_all(home)
    for sub in ["identities", "drafts", "inbox", "outbox-staged", "logs", "users"]:
        fs::create_dir_all(home/sub)
    Store::open_for_home_dir(home)   # создаёт liveletters.sqlite3
    SecretBox::open_or_create(default_key_path(home))  # mail-password-obfuscation.key
```

## Структура файлов

```
modules/liveletters-init/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── args.rs        # Args { force }
│   ├── error.rs       # InitError (thiserror)
│   └── run.rs         # run, run_inner, ensure_home_empty
└── tests/
    ├── common/mod.rs  # init_ctx() helper
    └── init_flow.rs   # 4 unit-теста
```

## Поведение `ensure_home_empty`

| Состояние `home` | `force` | Результат |
|---|---|---|
| не существует | `false`/`true` | OK (создаём с нуля) |
| существует, пуст | `false`/`true` | OK (заполняем) |
| существует, не пуст | `false` | `Err(InitError::AlreadyExists(home))` |
| существует, не пуст | `true` | OK (создаём недостающее; существующие файлы не удаляются) |

Решение «не пуст» принимается по наличию хотя бы одной записи в `home.read_dir()?.next()`. Скрытые файлы (`.foo`) тоже считаются не-пустотой — это сделано намеренно, чтобы `init` не молча перезаписывал пользовательские `.git`, `.DS_Store` и т.п.

## `mail-password-obfuscation.key`

Файл ключа создаётся через `SecretBox::open_or_create`, который:

- если файла нет — генерирует 32-байтный ключ через `OsRng` и пишет его с правами 0o600 (на Unix);
- если файл есть — открывает его, валидируя длину.

Файл лежит в `home` рядом с `liveletters.sqlite3` (имя задаётся `liveletters_secret_box::default_key_path`).

## Зависимости

| Крейт | Зачем |
|---|---|
| `liveletters-output` | `CommandContext` (единая точка прокидывания home/identity) |
| `liveletters-secret-box` | создание ключа обфускации |
| `liveletters-store` | инициализация SQLite-БД |
| `clap` (derive) | парсинг `--force` |
| `thiserror` | derive `Error` для `InitError` |

## Тесты

Юнит-тесты в `modules/liveletters-init/tests/init_flow.rs`:

- `init_creates_expected_layout` — после `init` существуют служебные подкаталоги, `liveletters.sqlite3` и `mail-password-obfuscation.key`;
- `init_is_idempotent_when_home_empty` — повторный `init` на пустом каталоге отрабатывает без ошибок;
- `init_fails_when_home_not_empty_without_force` — каталог с произвольным файлом → `InitError::AlreadyExists`;
- `init_force_overwrites_existing_files` — с `--force` каталог с посторонним файлом инициализируется, а файл остаётся на диске.

Интеграционные тесты в `apps/lltt/tests/cli_init.rs` (запускают бинарь `lltt`):

- `init_creates_liveletters_sqlite3` — после `lltt init` файл БД существует;
- `init_fails_on_non_empty_home_without_force` — `lltt init` возвращает код ошибки;
- `init_force_succeeds_on_non_empty_home` — `lltt init --force` отрабатывает на непустом каталоге.

## Что осталось за пределами текущей версии

- `init` не проверяет, что внутри `home` нет посторонних identity-файлов; если они есть, они остаются нетронутыми (это осознанно: пользователь мог вручную положить `alice.toml` перед первым `init`).
- `init` не валидирует `mail-password-obfuscation.key` при инициализации — файл может быть повреждён, и это обнаружится только при первом обращении к SMTP/IMAP-паролю.
- `init` не поддерживает `init --as <имя>` для первичной настройки конкретной идентичности.
