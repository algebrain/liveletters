# `liveletters-inbox` — TECHNICAL_SPEC

## 1. Цель

Крейт реализует команду `lltt inbox`. Содержит три подкоманды:

- `import` — превращает `raw .eml` в `SyncMessageOutcome` через цепочку
  `parse_email → ReceivedEmail → SyncEngine::ingest_batch → SyncReport`.
  Закрывает левую половину стержневого сценария: «электронное письмо → БД».
- `list` — показывает последние N строк таблицы `raw_messages` с
  фильтром по статусу (`--status` / `--limit`). Это второй уровень
  диагностики после агрегатов `lltt doctor`.
- `show` — печатает полное тело одного письма из `raw_messages`
  (для диагностики `Malformed`-писем).

## 2. Архитектура и зависимости

```
apps/lltt
   └─ clap-разбор → liveletters_inbox::Args
                       └─ run(ctx, args)
                            └─ match action:
                                 InboxAction::Import(import_args)
                                   └─ import::run(&ctx.home, &files)
                                        ├─ Store::open_for_home_dir
                                        ├─ SyncEngine::new(&store)
                                        └─ for each file:
                                             ├─ read_to_string
                                             ├─ parse_email → message_id + raw
                                             ├─ ReceivedEmail { message_id, raw_message }
                                             ├─ engine.ingest_batch(vec![email])
                                             └─ match each outcome → println! + счётчик
                                 InboxAction::List(list_args)
                                   └─ list::run(&ctx.home, &list_args)
                                        ├─ Store::open_for_home_dir
                                        ├─ list_raw_message_records           (для счётчика «всего»)
                                        ├─ list_raw_message_records_paged     (WHERE + ORDER BY rowid DESC LIMIT ?)
                                        └─ print_kv + print_table
                                 InboxAction::Show(show_args)
                                   └─ show::run(&ctx.home, &show_args)
                                        ├─ Store::open_for_home_dir
                                        ├─ get_raw_message_record(id)
                                        └─ print_message(message_id, status, raw_message)
```

Зависимости (`Cargo.toml`): `liveletters-mime`, `liveletters-output`,
`liveletters-store`, `liveletters-sync`, `clap` (с `derive`), `thiserror`.

## 3. Структура модуля

| Файл | Назначение |
|---|---|
| `src/args.rs` | `Args { action: InboxAction }`, `InboxAction::{Import, List, Show}`, `ImportArgs { files }`, `ListArgs { status, limit }`, `ShowArgs { id }` через `clap`. |
| `src/error.rs` | `InboxError { Store, Mime, Sync, Io, FileNotFound, InvalidStatus, MessageNotFound }` через `thiserror`. |
| `src/import.rs` | `run(home, files)` — главная логика импорта. |
| `src/list.rs` | `run(home, list_args)` — выборка из `raw_messages` + таблица. |
| `src/show.rs` | `run(home, show_args)` — печать тела одного письма. |
| `src/run.rs` | диспетчер по `InboxAction`. |
| `src/lib.rs` | реэкспорт + `summary() = "управление входящей почтой"`. |

## 4. Цепочка обработки одного `.eml`

1. `fs::read_to_string(file)` — читает файл в UTF-8.
2. `parse_email(&raw)` (`liveletters-mime`) — вытаскивает заголовки и тело; ожидается письмо LiveLetters с человекочитаемым текстом и служебным блоком.
3. Извлечение `message_id` из заголовка `Message-ID` (или `Message-Id` — MIME регистронезависим). Если заголовка нет — пустая строка; это не ошибка (sync всё равно попробует применить).
4. `SyncEngine::ingest_batch(vec![ReceivedEmail { message_id, raw_message: raw }])`.
5. На каждый `SyncMessageOutcome` — печать строки и инкремент соответствующего счётчика.

## 5. `message_id` как ключ идемпотентности

`ReceivedEmail.message_id` используется `SyncEngine` для дедупликации. Если в БД уже есть `raw_messages` с тем же `message_id` — событие отбрасывается как `Duplicate`. Если `message_id` пуст (например, в `.eml` нет заголовка) — `SyncEngine` всё равно может применить, если в БД нет точно такого же `raw_message`. Поведение согласовано с импортом через IMAP.

## 6. Категории исходов

Сводная таблица в `INTERFACE.md` (раздел «Категории исходов»). Кратко: `Applied` — попало в БД; `Duplicate` — уже было; `Deferred` — отложено из-за неполноты состояния; `Filtered` — отфильтровано подписками; `Malformed`/`Replay`/`Unauthorized`/`Invalid` — отклонено с указанием причины.

## 7. Алгоритм `list`

1. Валидация `args.status`: значение должно входить в `ALLOWED_STATUSES`:
   `applied`, `duplicate`, `replay`, `unauthorized`, `invalid`, `malformed`.
   Иначе — `InboxError::InvalidStatus`.
2. `Store::open_for_home_dir(home)`.
3. `store.list_raw_message_records().len()` — для счётчика «входящих всего».
4. `store.list_raw_message_records_paged(args.status.as_deref(), args.limit)` —
   SQL `WHERE status = ? ORDER BY rowid DESC LIMIT ?` (новые сверху).
5. `print_kv` со сводкой (всего / фильтр / показано), затем `print_table`
   с колонками `message_id`, `status`, `preview`.
6. Если `list_raw_message_records_paged` вернул пусто — печатается `(пусто)`.

`preview` — первая непустая строка `raw_message` (обычно тема письма)
длиной до 80 символов; более длинные строки обрезаются с `…`.

## 8. Алгоритм `show`

1. `Store::open_for_home_dir(home)`.
2. `store.get_raw_message_record(&args.id)` — SQL
   `SELECT ... FROM raw_messages WHERE message_id = ?`.
3. Если `Ok(None)` — `InboxError::MessageNotFound(args.id)`.
4. Иначе — `print_message(message_id, status, raw_message)`:
   ```
   message_id: <id>
   status: <status>

   --- body ---
   <raw_message>
   ```

## 9. Что НЕ делает

- Не подключается к IMAP.
- Не отправляет по SMTP.
- Не применяет фильтры на уровне команды (фильтры работают внутри `SyncEngine` по `meta.subscriptions`).
- `show` не поддерживает несколько id и не фильтрует по статусу.

## 10. Сценарии ошибок

| Сценарий | Возврат |
|---|---|
| Файл не существует | `InboxError::FileNotFound(file)` |
| Не UTF-8 | `InboxError::Io(io::Error)` |
| Нет служебного блока LiveLetters | `InboxError::Mime(MissingTechnicalPart)` |
| Нет `current-user` (нет `init`) | `InboxError::Store(StoreError::StoreNotInitialized)` |
| `list --status nonsense` | `InboxError::InvalidStatus("nonsense")` |
| `show <unknown_id>` | `InboxError::MessageNotFound("<unknown_id>")` |
| Событие с тем же `event_id` уже применено | `total_duplicate += 1`, без ошибки |
| Событие невалидно | `total_rejected += 1`, без ошибки (например, отсутствует `event_id`) |

## 11. Совместимость

- В ранних версиях `Args` был пустой, `run` возвращал `NotYetImplemented`.
- Потом: `InboxAction::Import` + `import::run` + сводный отчёт.
- Затем: `InboxAction::List(ListArgs { status, limit })` + `list::run`.
- Сейчас: добавлен `InboxAction::Show(ShowArgs { id })` + `show::run`;
  `list` переведён с `list_raw_message_records` + reverse/take
  на `list_raw_message_records_paged` (SQL-уровень).
- Запланировано: реальный IMAP через `liveletters-mail::ConfiguredImapMailbox`; `import::run` остаётся как fallback для оффлайнового импорта.
