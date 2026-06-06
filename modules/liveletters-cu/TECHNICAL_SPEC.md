# `liveletters-cu` TECHNICAL_SPEC

## Назначение

`liveletters-cu` содержит реализацию двух CLI-веток:

- `run_current` для `lltt cu`;
- `run_user` для `lltt user`.

Разделение сделано на уровне функций входа, а не отдельного крейта: обе ветки используют один тип `Args`, один тип ошибок, одну модель `IdentityConfig` и одни операции над домашним каталогом.

## Модули

```
modules/liveletters-cu/src/
├── lib.rs
├── args.rs                  # Args и CuAction
├── run.rs                   # разбор tokens, run_current, run_user
├── current.rs               # печать текущего пользователя
├── switch.rs                # запись current-user
├── list.rs                  # identities/*.toml
├── show.rs                  # печать IdentityConfig
├── add.rs                   # добавление IdentityConfig и mail_settings
├── rm.rs                    # удаление идентичности
├── user_init.rs             # создание drafts/<name>.toml
├── name.rs                  # проверка имени
├── password_obfuscation.rs  # подтверждение и скрытие паролей
└── error.rs
```

## Разбор `lltt cu`

`parse_current_action(tokens)` возвращает только действия текущего пользователя:

| `tokens` | Действие |
|---|---|
| `[]` | `CuAction::Current` |
| `["show"]` | `CuAction::ShowCurrent { reveal: false }` |
| `["show", "--reveal"]` | `CuAction::ShowCurrent { reveal: true }` |
| `["posts"]` | `CuAction::Posts { limit: None }` |
| `["posts", "--limit", "20"]` | `CuAction::Posts { limit: Some(20) }` |
| `["posts", "--limit=20"]` | `CuAction::Posts { limit: Some(20) }` |
| `["alice"]` | `CuAction::Switch { name: "alice" }` |

Если первый токен равен `list`, `add`, `rm` или `show` с именем, возвращается `CuError::UseUserCommand(...)`. Если после имени переключения есть дополнительные токены, возвращается `CuError::ConflictingArgs`.

## Разбор `lltt user`

`parse_user_action(tokens)` принимает только подкоманды:

| `tokens` | Действие |
|---|---|
| `["list"]` | `CuAction::List` |
| `["init", "alice"]` | `CuAction::Init { name: "alice", force: false }` |
| `["init", "alice", "--force"]` | `CuAction::Init { name: "alice", force: true }` |
| `["show", "alice"]` | `CuAction::Show { name: "alice", reveal: false }` |
| `["show", "alice", "--reveal"]` | `CuAction::Show { name: "alice", reveal: true }` |
| `["add", "alice"]` | `CuAction::Add { name: "alice", from: None }` |
| `["add", "alice", "--from", "drafts/alice.toml"]` | `CuAction::Add { name: "alice", from: Some(...) }` |
| `["rm", "alice", "--yes"]` | `CuAction::Rm { name: "alice", yes: true }` |

Отсутствующая подкоманда, неизвестный флаг или лишний позиционный аргумент возвращают `CuError::InvalidArgs`.

## Выполнение

`run_current`:

1. разбирает токены через `parse_current_action`;
2. `Current` вызывает `current::run`;
3. `Switch` вызывает `switch::run`, который проверяет наличие файла идентичности, создаёт локальное состояние пользователя и пишет `current-user`;
4. `ShowCurrent` читает `current-user`, затем вызывает `show::run`.
5. `Posts` вызывает `liveletters_posts::run` с разобранным `limit`.

`run_user`:

1. разбирает токены через `parse_user_action`;
2. `List` вызывает `list::run`;
3. `Init` вызывает `user_init::run`;
4. `Show` вызывает `show::run`;
5. `Add` вызывает `add::run`; если `--from` не указан, передаёт `<home>/drafts/<name>.toml`;
6. `Rm` вызывает `rm::run`.

## `user init`

`user_init::run(ctx, name, force)`:

1. проверяет имя через `validate_user_name`;
2. создаёт каталог `<home>/drafts`;
3. формирует путь `<home>/drafts/<name>.toml`;
4. если файл существует и `force == false`, возвращает `CuError::InvalidArgs`;
5. пишет черновик и печатает путь с содержимым.

Черновик включает `[mail.smtp]` и `[mail.imap]` с `pwd_obfuscate = true`, чтобы пользователь мог сразу заполнить почту, а `user add` перенёс её в `mail_settings`.

## `user add`

`add::run(ctx, name, from)`:

1. проверяет имя;
2. проверяет наличие файла `from`;
3. читает и разбирает TOML в `IdentityConfig`;
4. вызывает `obfuscate_identity_passwords`;
5. если пароли были скрыты, переписывает исходный TOML уже с `obf:v1:...`;
6. сохраняет идентичность через `save_identity`;
7. открывает `Store::open_for_home_dir(<home>/users/<name>)`;
8. копирует SMTP/IMAP-настройки из идентичности в `MailSettingsRecord` этого пользователя;
9. сохраняет `mail_settings`.

Команда намеренно не пишет `current-user`: выбор текущего пользователя остаётся отдельным явным шагом `lltt cu <имя>`.

## Скрытие паролей

`password_obfuscation.rs` содержит:

- `PasswordConfirmer` — интерфейс подтверждения;
- реализацию для терминала со скрытым вводом и звёздочками;
- `obfuscate_identity_passwords(home, cfg, confirmer)`.

Алгоритм обрабатывает SMTP и IMAP отдельно. Для каждой секции пароль скрывается только при `pwd_obfuscate = true`, непустом пароле и отсутствии префикса `obf:v1:`. Для `user add` ключ скрытия берётся из пользовательского состояния `<home>/users/<name>/`. Перед скрытием пользователь должен повторить пароль. Несовпадение даёт `CuError::PasswordConfirmationMismatch`.

## Удаление

`rm::run(ctx, name, yes)`:

1. требует `yes == true`;
2. если `<home>/current-user` существует и равен `name`, возвращает `CannotRemoveCurrent`;
3. удаляет `<home>/identities/<name>.toml`.

Отсутствующий `current-user` не мешает удалить невыбранную идентичность.

## Тесты

- `modules/liveletters-cu/tests/cu_flow.rs` проверяет старые базовые операции крейта: текущий пользователь, переключение, список, показ, добавление, удаление и маскирование вывода.
- Юнит-тесты `password_obfuscation` проверяют скрытие SMTP/IMAP-паролей, несовпадение подтверждения и отключённое `pwd_obfuscate`.
- `apps/lltt/tests/cli_cu.rs` проверяет CLI-поведение `cu`, `cu posts`, запрет старых форм управления списком и подсказки перейти на `user`.
- `apps/lltt/tests/cli_user.rs` проверяет создание черновика, `--force`, добавление пользователя, перенос почты в `mail_settings` и отсутствие автоматического выбора текущего пользователя.

## Ограничения

- `user add` не проверяет доступность SMTP/IMAP-серверов; он только валидирует TOML и сохраняет настройки.
- `pwd_obfuscate` принимает логическое значение TOML (`true`/`false`), а не строковые значения.
- `cu show` показывает только текущего пользователя; просмотр произвольной идентичности находится в `user show`.
