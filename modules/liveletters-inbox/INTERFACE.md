# `liveletters-inbox` — INTERFACE

## Назначение

`liveletters-inbox` — библиотечный крейт, реализующий команду `lltt inbox`.
Содержит три подкоманды: `import` (импорт `.eml` через `SyncEngine`),
`list` (просмотр таблицы `raw_messages` с фильтром по статусу) и
`show` (печать полного тела одного письма).

## Где находится интерфейс

- crate: `liveletters-inbox`
- точка подключения: [`src/lib.rs`](src/lib.rs)
- алгоритм `run`: [`src/run.rs`](src/run.rs)
- логика импорта: [`src/import.rs`](src/import.rs)
- логика списка: [`src/list.rs`](src/list.rs)
- логика показа: [`src/show.rs`](src/show.rs)

## Публичный API

```rust
pub use args::{Args, ImportArgs, InboxAction, ListArgs, ShowArgs};
pub use error::InboxError;
pub use run::run;
pub use liveletters_output::CommandContext;

pub const COMMAND_NAME: &str;
pub fn summary() -> &'static str;
pub fn crate_name() -> &'static str;

pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

## Подкоманды

`lltt inbox` имеет **три** подкоманды: `import`, `list` и `show`.

```
lltt inbox import <файл.eml> [<файл.eml>…]
lltt inbox list [--status <категория>] [--limit <N>]
lltt inbox show <message_id>
```

### `import`

```rust
#[derive(Debug, clap::Args)]
pub struct ImportArgs {
    /// Один или несколько .eml-файлов для импорта.
    #[arg(required = true)]
    pub files: Vec<std::path::PathBuf>,
}
```

`files` — позиционные аргументы; минимум один.

### `list`

```rust
#[derive(Debug, clap::Args)]
pub struct ListArgs {
    /// Фильтр по статусу: applied, duplicate, replay, unauthorized, invalid, malformed.
    #[arg(long)]
    pub status: Option<String>,
    /// Сколько последних писем показать (по умолчанию 20).
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}
```

`--status` принимает строго одно из шести значений. Прочие значения
отклоняются ошибкой `InboxError::InvalidStatus`.

`--limit` ограничивает размер выдачи; значение по умолчанию — 20.
Сортировка — новые сверху (`ORDER BY rowid DESC` на уровне SQL).

### `show`

```rust
#[derive(Debug, clap::Args)]
pub struct ShowArgs {
    /// Идентификатор сообщения (значение `Message-ID` или `message_id` в БД).
    pub id: String,
}
```

Печатает `message_id`, `status` и полное тело (`raw_message`) одного
письма из таблицы `raw_messages`. При отсутствии — `InboxError::MessageNotFound`
(код 1). `message_id` — строковое значение с угловыми скобками
(например, `<p-1@example.test>`), как его положил MIME-парсер.

## `InboxError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum InboxError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка MIME-разбора: {0}")]
    Mime(#[from] liveletters_mime::MimeError),

    #[error("ошибка синхронизации: {0}")]
    Sync(#[from] liveletters_sync::SyncError),

    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("файл {0} не найден")]
    FileNotFound(std::path::PathBuf),

    #[error("неизвестный статус: {0}; допустимые: applied, duplicate, replay, unauthorized, invalid, malformed")]
    InvalidStatus(String),

    #[error("сообщение с id {0} не найдено в raw_messages")]
    MessageNotFound(String),
}
```

## Алгоритм `import`

1. Открыть `Store::open_for_home_dir(&ctx.home)`.
2. Создать `SyncEngine::new(&store)`.
3. Для каждого файла из `args.files`:
   - проверить `file.exists()` → иначе `InboxError::FileNotFound(file)`;
   - `fs::read_to_string(file)` → `raw_message`;
   - `parse_email(&raw)` → `ParsedEmail` → `message_id` берётся из заголовка `Message-ID` (или `Message-Id`); если его нет — пустая строка;
   - собрать `ReceivedEmail { message_id, raw_message }`;
   - `engine.ingest_batch(vec![received])` → `SyncReport`;
   - напечатать построчно исход для каждого `SyncMessageOutcome` (см. ниже);
   - накопить счётчики по категориям.
4. Напечатать итоговую сводку.

## Алгоритм `list`

1. Если `args.status` задан — проверить, что он входит в `ALLOWED_STATUSES`
   (`applied`, `duplicate`, `replay`, `unauthorized`, `invalid`, `malformed`).
   Иначе — `InboxError::InvalidStatus`.
2. Открыть `Store::open_for_home_dir(&ctx.home)`.
3. `store.list_raw_message_records()` — для подсчёта общего числа строк.
4. `store.list_raw_message_records_paged(args.status.as_deref(), args.limit)` —
   SQL-запрос `ORDER BY rowid DESC LIMIT ?` (новые сверху).
5. Напечатать `print_kv`-сводку (всего / фильтр / показано) и таблицу
   из трёх колонок: `message_id`, `status`, `preview`.
6. Если после фильтра пусто — напечатать `(пусто)`.

`preview` — первая непустая строка `raw_message` (обычно тема письма)
длиной до 80 символов; более длинные строки обрезаются с `…`.

## Алгоритм `show`

1. `Store::open_for_home_dir(&ctx.home)`.
2. `store.get_raw_message_record(&args.id)` — SQL
   `SELECT ... WHERE message_id = ?`.
3. Если `Ok(None)` — `InboxError::MessageNotFound(args.id)`.
4. Иначе — `print_message(message_id, status, raw_message)`:
   ```
   message_id: <id>
   status: <status>

   --- body ---
   <raw_message>
   ```

## Категории исходов `SyncMessageOutcome`

| Вариант | Печатаемая строка | Счётчик |
|---|---|---|
| `Applied { event_id, .. }` | `<файл>: применено (<event_id>)` | `total_applied` |
| `Duplicate { event_id, .. }` | `<файл>: дубликат (<event_id>)` | `total_duplicate` |
| `Deferred { reason, .. }` | `<файл>: отложено (<reason>)` | `total_deferred` |
| `Filtered { reason, .. }` | `<файл>: отфильтровано (<reason>)` | `total_filtered` |
| `Malformed { reason, .. }` | `<файл>: отклонено (<reason>)` | `total_rejected` |
| `Replay { reason, .. }` / `Unauthorized { reason, .. }` / `Invalid { reason, .. }` | `<файл>: отклонено (<reason>)` | `total_rejected` |

Финальная сводка `import`:

```
применено: <N>
дубликатов: <N>
отложено:   <N>
отфильтровано: <N>
отклонено:  <N>
```

## Соседи

- [`liveletters-mime`](../../modules/liveletters-mime/INTERFACE.md) — `parse_email`, `build_protocol_email`, `ReceivedEmail`, `MimeError`.
- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — `Store::open_for_home_dir`, `list_raw_message_records`, `list_raw_message_records_paged`, `get_raw_message_record`, `RawMessageRecord`.
- [`liveletters-sync`](../../modules/liveletters-sync/INTERFACE.md) — `SyncEngine::new`, `ingest_batch`, `SyncReport`, `SyncMessageOutcome`, `SyncError`.
- [`liveletters-feed`](../../modules/liveletters-feed/INTERFACE.md) — потребитель результатов: после `inbox import` пользователь открывает `lltt feed`.
- [`liveletters-doctor`](../../modules/liveletters-doctor/INTERFACE.md) — агрегированные счётчики по статусам; `list` показывает отдельные строки, `show` — тело одной строки.

## Тесты

- `tests/import.rs` (4 теста) — `import::run` с валидным/повторным/отсутствующим/повреждённым `.eml`.
- `tests/list.rs` (4 теста) — `list::run` на пустой БД, с фильтром, с лимитом, с невалидным статусом.
- `tests/show.rs` (3 теста) — `show::run` на существующий id, на несуществующий, на пустую строку.
- `tests/common/mod.rs` — фикстура `write_valid_post_eml` строит корректное multipart-сообщение через `build_protocol_email`.
- Покрытие через бинарь: `apps/lltt/tests/cli_inbox_import.rs` (3 теста), `apps/lltt/tests/cli_inbox_list.rs` (5 тестов), `apps/lltt/tests/cli_inbox_show.rs` (2 теста).

