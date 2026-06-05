# `liveletters-cu` INTERFACE

## Назначение

`liveletters-cu` реализует две команды:

- `lltt cu` — работа только с текущим пользователем liveletters;
- `lltt user` — управление списком идентичностей и черновиками.

Крейт оставлен один, без отдельного `liveletters-user`, потому что обе команды используют одну модель идентичности, один разбор TOML и одни операции над каталогами `identities/`, `drafts/` и файлом `current-user`.

## Публичный интерфейс

- crate: `liveletters-cu`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- `Args` — clap-аргументы (`tokens: Vec<String>`, режим `trailing_var_arg`);
- `CuAction` — внутреннее перечисление действий;
- `CuError` — типизированные ошибки;
- `CommandContext` — реэкспорт из `liveletters-output`;
- `run_current(&CommandContext, &Args)` — вход для `lltt cu`;
- `run_user(&CommandContext, &Args)` — вход для `lltt user`;
- `run(&CommandContext, &Args)` — совместимый вход, равен `run_current`;
- `COMMAND_NAME: &str = "cu"`;
- `summary()`, `crate_name()`.

## `lltt cu`

`cu` означает «текущий пользователь». Команда не управляет списком идентичностей.

| Форма | Действие |
|---|---|
| `lltt cu` | Печатает имя из `<home>/current-user`. Требует выбранного пользователя. |
| `lltt cu <имя>` | Проверяет наличие `<home>/identities/<имя>.toml` и записывает `<имя>` в `<home>/current-user`. |
| `lltt cu show [--reveal]` | Печатает идентичность текущего пользователя. Пароли маскируются, кроме режима `--reveal`. |

Старые формы `lltt cu list`, `lltt cu show <имя>`, `lltt cu add ...`, `lltt cu rm ...` запрещены. Они возвращают `CuError::UseUserCommand` с подсказкой перейти на соответствующую форму `lltt user ...`.

## `lltt user`

`user` управляет идентичностями. Команда работает сразу после `lltt init`, даже если текущий пользователь ещё не выбран.

| Форма | Действие |
|---|---|
| `lltt user list` | Печатает имена файлов `identities/*.toml` по одному в строке. |
| `lltt user init <имя> [--force]` | Создаёт черновик `<home>/drafts/<имя>.toml`, печатает путь и содержимое. Без `--force` не перезаписывает существующий черновик. |
| `lltt user show <имя> [--reveal]` | Загружает `<home>/identities/<имя>.toml` и печатает его через `liveletters_output::print_identity`. |
| `lltt user add <имя> --from <путь>` | Читает TOML, проверяет имя, скрывает пароли при `pwd_obfuscate = true`, сохраняет `<home>/identities/<имя>.toml` и копирует почтовые секции в `mail_settings`. Текущего пользователя не меняет. |
| `lltt user rm <имя> --yes` | Удаляет `<home>/identities/<имя>.toml`; без `--yes` возвращает ошибку; текущего пользователя удалить нельзя. |

## Черновик идентичности

`lltt user init alice` создаёт TOML такого вида:

```toml
account_id = "acct_alice"
display_name = "Alice"

[mail]
publish = "alice@example.org"
receive = ["alice@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "alice@example.org"
password = ""
pwd_obfuscate = true
hello_domain = "example.org"

[mail.imap]
host = "imap.example.org"
port = 993
security = "tls"
username = "alice@example.org"
password = ""
pwd_obfuscate = true
mailbox = "INBOX"

[meta]
resources_owned = ["alice@example.org"]
subscriptions = []
```

Имя не может быть пустым, `.` или `..`, содержать пробелы, `/` или `\`.

## Пароли

При `lltt user add` пароль скрывается только если выполнены все условия:

- пароль непустой;
- пароль ещё не начинается с `obf:v1:`;
- в соответствующей секции стоит `pwd_obfuscate = true`.

SMTP- и IMAP-пароли подтверждаются отдельно. Ввод скрытый: на экране печатаются звёздочки. Если подтверждение не совпало, команда завершается ошибкой и не сохраняет изменённый пароль.

Скрытие выполняется через `liveletters-secret-box::SecretBox` и ключ `<home>/mail-password-obfuscation.key`.

## Ошибки

`CuError` покрывает:

- ошибки конфигурации (`Config`);
- ошибки ввода-вывода (`Io`);
- отсутствующий файл для `--from`;
- попытку удалить текущего пользователя;
- неверные аргументы;
- конфликт позиционных аргументов;
- подсказку перейти с устаревшей формы `cu` на `user`;
- ошибку хранилища при записи `mail_settings`;
- ошибку скрытия секрета;
- ошибку скрытого ввода пароля;
- несовпадение подтверждения пароля.

Все варианты реализуют `std::error::Error`.

## Побочные эффекты

- `lltt cu <имя>` пишет только `<home>/current-user`.
- `lltt user init` пишет только `<home>/drafts/<имя>.toml`.
- `lltt user add` пишет `<home>/identities/<имя>.toml`, может переписать исходный TOML с уже скрытыми паролями и сохраняет `mail_settings` в БД.
- `lltt user rm` удаляет файл идентичности.
- `lltt user add` не выбирает пользователя текущим.

## Зависимости

- `liveletters-config` — чтение и запись идентичностей, `current-user`;
- `liveletters-store` — сохранение `mail_settings`;
- `liveletters-secret-box` — скрытие паролей;
- `liveletters-output` — `CommandContext`, печать идентичности;
- `console` — скрытый ввод со звёздочками;
- `toml`, `thiserror`, `clap`.
