# `liveletters-output` INTERFACE

## Назначение

`liveletters-output` собирает общие функции человекочитаемого вывода, которыми пользуются командные крейты `lltt`. Выделение в отдельную библиотеку сделано, чтобы команды (`init`, `cu`, `home`, `feed`, и т.д.) могли единообразно печатать результат и маскировать секреты без копирования кода и без зависимости от бинаря `apps/lltt`.

## Где находится интерфейс

- crate: `liveletters-output`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- структура `CommandContext` — минимальный контекст команды (`{ home: PathBuf, identity_name: String }`);
- функция `mask_password(plain: &str, reveal: bool) -> String` — единая точка маскирования секретов в выводе;
- функция `print_kv(pairs: &[(&str, &str)])` — печать пар «ключ: значение»;
- функция `print_table(headers: &[&str], rows: &[Vec<String>])` — печать таблицы с выравниванием колонок;
- функция `print_identity(cfg: &IdentityConfig, reveal: bool)` — печать секций `[identity]`, `[mail]`, `[mail.smtp]`, `[mail.imap]`, `[resources_owned]` с маскированием SMTP/IMAP-паролей;
- `crate_name() -> &'static str` — имя креЙта, для диагностических сообщений и тестов.

Внутренние модули `context` и `format` не публикуются.

## `CommandContext`

```rust
pub struct CommandContext {
    pub home: PathBuf,
    pub identity_name: String,
}
```

Контекст передаётся первым аргументом в `pub fn run(&CommandContext, &Args) -> Result<(), Box<dyn Error + Send + Sync>>` каждого командного креЙта. Содержит:

- `home` — путь к домашнему каталогу `lltt` (значение переменной `LIVELETTERS_HOME` или `<user-home>/.liveletters/`, см. `liveletters-store::resolve_data_dir_from_env`);
- `identity_name` — имя текущего пользователя liveletters (читается из файла `<home>/current-user` через `liveletters_config::read_current_identity`). В коде называется `identity_name` (исторически), в публичной документации — «текущий пользователь liveletters».

## `mask_password`

```rust
pub fn mask_password(plain: &str, reveal: bool) -> String
```

- `reveal == true` — возвращает `plain` как есть (включая пустую строку).
- `reveal == false` — **всегда** возвращает литерал `"********"` (8 звёздочек), в том числе для пустого ввода.

Семантика маски зафиксирована двумя тестами в `liveletters-output/tests/mask.rs`:
- `mask_password_returns_masks_when_hidden` — `mask_password("hunter2", false) == "********"` (нельзя случайно изменить формат маски);
- `mask_password_empty_returns_masks_or_plain` — пустой ввод при `reveal=false` тоже даёт `********` (команда не различает «пароль не задан» и «пароль задан»).

## `print_kv`

```rust
pub fn print_kv(pairs: &[(&str, &str)])
```

Печатает каждую пару на отдельной строке в формате `key: value` с `\n` после каждой. Используется внутри `print_identity` и доступна командам, которым нужно напечатать плоский список атрибутов.

## `print_table`

```rust
pub fn print_table(headers: &[&str], rows: &[Vec<String>])
```

Печатает таблицу с заголовками. Колонки выравниваются по ширине самой длинной ячейки (включая заголовок), разделитель — два пробела. Не использует `tui`/`crossterm`, пишет в `stdout` построчно.

## `print_identity`

```rust
pub fn print_identity(cfg: &IdentityConfig, reveal: bool)
```

Печатает идентичность в формате:

```
[identity]
account_id: <id>
display_name: <name>

[mail]
publish: <url>
receive: [0] <addr-0>
receive: [1] <addr-1>

[mail.smtp]
host: <host>
port: <port>
security: <tls|starttls|none>
username: <user>
password: <******** или plain>

[mail.imap]
... аналогично ...

[resources_owned]
- <id>
```

Секции `[mail.smtp]` и `[mail.imap]` печатаются, только если соответствующая настройка присутствует в `cfg.mail().smtp()` / `cfg.mail().imap()`. Пароли маскируются через `mask_password` с флагом `reveal`.

## Зависимости

- `liveletters-config` — для типа `IdentityConfig` и связанных структур.

## Пример использования

```rust
use liveletters_config::load_identity;
use liveletters_output::{print_identity, CommandContext};
use std::path::PathBuf;

let home = PathBuf::from("/var/lib/lltt");
let ctx = CommandContext { home: home.clone(), identity_name: "alice".to_owned() };
let cfg = load_identity(&ctx.home, &ctx.identity_name)?;
print_identity(&cfg, false); // пароли скрыты
```
