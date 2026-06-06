# `liveletters-output` TECHNICAL_SPEC

## Назначение

`liveletters-output` — общий крейт вывода для команд `lltt`. Не делает бизнес-логики, не зависит от `liveletters-store`, `liveletters-mail` или сетевых крейтов; только форматирует уже готовые значения и маскирует секреты.

## Зона ответственности

- хранение `CommandContext` (`home`, `state_home`, `identity_name`);
- единая точка маскирования паролей в stdout;
- печать плоских списков «ключ-значение»;
- печать таблиц с выравниванием;
- печать полной идентичности в секционном виде.

## Что крейт не должен делать

- не открывает БД, не трогает файлы, не делает сетевых вызовов;
- не решает, какие команды и в каком порядке запускать (это задача `apps/lltt/src/main.rs`);
- не принимает решений о видимости секретов — флаг `reveal` приходит от пользователя;
- не логирует, не пишет в stderr (только stdout).

## Структура файлов

```
modules/liveletters-output/
├── Cargo.toml
├── src/
│   ├── lib.rs          # re-exports + crate_name
│   ├── context.rs      # CommandContext
│   └── format.rs       # mask_password, print_kv, print_table, print_identity
└── tests/
    └── mask.rs         # тесты маскирования и форматирования
```

## `CommandContext`

```rust
pub struct CommandContext {
    pub home: PathBuf,
    pub state_home: PathBuf,
    pub identity_name: String,
}
```

Контекст не хранит «текущего пользователя операционной системы» и не пытается определить, кто запустил процесс. `identity_name` приходит из файла `<home>/current-user` (см. `liveletters_config::read_current_identity`). `state_home` вычисляется из него как `<home>/users/<identity_name>` и задаёт отдельную локальную БД текущего пользователя. Это сознательное решение: ни переменная окружения `LLTT_CU`, ни флаг `--as` сейчас **не поддерживаются** — иначе состояние «выбран через `lltt cu`, но `LLTT_CU` указывает на другое» становилось бы неотлаживаемым.

## `mask_password`

Логика строго фиксирована:

| `plain` | `reveal` | результат |
|---|---|---|
| `""` | `true` или `false` | `""` |
| не пуст | `true` | `plain` |
| не пуст | `false` | `"********"` (8 ASCII-звёздочек) |

Семантика «8 звёздочек для непустого» выбрана, чтобы:

- не зависеть от длины пароля (нельзя восстановить длину по выводу);
- не использовать Unicode-символы, чтобы вывод одинаково читался в разных локалях;
- дать стабильный токен для интеграционных тестов (substring `********` в stdout).

## `print_table`

Алгоритм:

1. вычислить максимальную ширину каждой колонки как `max(заголовок_i, max(ячейка_i по строкам))`, ширина считается в Unicode-символах (`chars().count()`);
2. напечатать строку заголовков, колонки разделены двумя пробелами, в каждой ячейке справа дополнение пробелами до ширины колонки;
3. напечатать пустую строку-разделитель (заголовок-данные);
4. напечатать строки данных тем же выравниванием.

Таблица не поддерживает многострочные ячейки, перенос строк и цвета. Это осознанное упрощение: целевой вывод — терминал 80–120 колонок, в котором такие фичи избыточны.

## `print_identity`

Алгоритм:

1. секция `[identity]`: `account_id`, `display_name`;
2. пустая строка;
3. секция `[mail]`: `publish`; для каждого адреса в `mail.receive` — отдельная строка `receive: [i] <addr>`; если список пуст — `receive: -`;
4. если `mail.smtp.is_some()` — пустая строка + секция `[mail.smtp]` с полями `host`/`port`/`security`/`username`/`password` (пароль через `mask_password(..., reveal)`);
5. если `mail.imap.is_some()` — пустая строка + секция `[mail.imap]` (аналогично + `mailbox`);
6. если `resources_owned` не пуст — пустая строка + секция `[resources_owned]` со списком `- <id>`.

Формат секций соответствует заголовкам TOML, чтобы визуально совпадать с выводом `cat identities/<name>.toml` (полезно при ручной диагностике).

## Зависимости

- `liveletters-config` — тип `IdentityConfig` и его метод-аксессоры (`account_id()`, `display_name()`, `mail()`, `resources_owned()`, и т.д.).

Никаких других зависимостей (ни `serde_json`, ни `clap`, ни `tokio`).

## Тесты

- `modules/liveletters-output/tests/mask.rs` — 6 тестов:
  - `mask_password_returns_plain_when_revealed` — `reveal=true` возвращает вход как есть;
  - `mask_password_returns_masks_when_hidden` — `reveal=false` для непустого входа → `********`;
  - `mask_password_empty_returns_masks_or_plain` — пустой вход: `reveal=true` → `""`, `reveal=false` → `********` (команда не различает «не задан» и «задан»);
  - `print_identity_masks_smtp_password_by_default` — `IdentityConfig` с SMTP-паролем, проверка `mask_password("secret123", false) == "********"`;
  - `print_identity_reveal_shows_smtp_password` — аналогично с `reveal = true`;
  - `print_table_aligns_columns` — smoke-тест, что печать таблицы не паникует;
- `modules/liveletters-output/src/format.rs::tests` — 4 теста на ту же `mask_password` (дублируют интеграционные, но в `lib`-бинаре): `mask_password_empty_returns_eight_stars`, `mask_password_non_empty_returns_eight_stars`, `mask_password_reveal_returns_plain`, `mask_password_reveal_empty_returns_empty`;
- unit-тест в `src/lib.rs::exposes_crate_name` — `crate_name() == "liveletters-output"`.

## Что осталось за пределами текущей версии

- `liveletters-output` пока не использует `serde_json::Value` для «красивого» вывода сложных структур (например, `subscriptions`) — это будет нужно, когда такие структуры появятся в выводе команд.
- Формат таблицы не поддерживает выравнивание по правому краю для числовых колонок — добавляется по запросу.
