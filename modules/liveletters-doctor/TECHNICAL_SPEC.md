# liveletters-doctor

## Назначение

`liveletters-doctor` — командный крейт `lltt doctor`. Печатает
`DiagnosticsSnapshot`, агрегированный `DiagnosticsReader` из крейта
`liveletters-diagnostics`. С `--verbose` дополнительно показывает
deferred-события, identities и размеры таблиц БД.

## Зона ответственности

Крейт отвечает за:

- открытие `Store` через `Store::open_for_home_dir(&ctx.home)`;
- конструирование `DiagnosticsReader` поверх `Store`;
- вызов `build_snapshot`;
- перевод категорий в человекочитаемые строки (Healthy/Degraded);
- печать через `print_kv`;
- чтение `<home>/identities/*.toml` и `<home>/current-user`;
- (verbose) `Store::list_deferred_events(10)` + `Store::table_size(...)` для 6 таблиц.

Крейт **не** отвечает за:

- добавление новых счётчиков (это делается в `liveletters-diagnostics`);
- показ бизнес-счётчиков (число постов/комментариев — это `liveletters-status`).

## Текущее состояние реализации

- `Args { verbose: bool }` (derive `Default`, флаги `--verbose`/`-v`);
- `DoctorError` — три варианта: `Store(StoreError)`, `Diagnostics(String)`, `Io(io::Error)` (через `?`);
- `print_doctor` — печатает 9 строк;
- `print_doctor_verbose` — 9 строк + 3 секции (deferred/identities/таблицы);
- `run` — открывает Store, строит snapshot, выбирает формат по `args.verbose`;
- 7 интеграционных тестов (4 в `flow.rs` + 3 в `verbose.rs`).

## Алгоритм `print_doctor_verbose`

1. `print_doctor(snap)` — стандартные 9 строк.
2. Секция `--- deferred_events (последние 10) ---`:
   - `store.list_deferred_events(10)`;
   - если пусто — `(нет)`, иначе `  - <event_id>: <reason>` построчно.
3. Секция `--- identities ---`:
   - читается `<home>/current-user` (если нет — `(не задан)`);
   - `fs::read_dir(<home>/identities)` → имена файлов с суффиксом `.toml`,
     отсортированные, без суффикса;
   - печатается `<N> конфигов: <список>; текущий: <current-user>`;
   - если каталог отсутствует — `(каталог identities/ отсутствует)`.
4. Секция `--- таблицы ---`:
   - для каждой таблицы из `[posts, comments, outbox, raw_messages, deferred_events, subscriptions]`
     вызывается `store.table_size(name)` (через `dbstat`);
   - при ошибке печатается `0 B`.

## Критерии готовности

- `cargo build -p liveletters-doctor` зелёный;
- `cargo test -p liveletters-doctor` зелёный (7 тестов);
- `lltt doctor` после `lltt init` печатает «здоровье: здоров» и нули во
  всех категориях;
- `lltt doctor --verbose` после `lltt init` дополнительно печатает три
  секции (deferred пуст, identities показывает текущего пользователя,
  таблицы содержат размеры ≤ 4 KiB).

## Связанные документы

- [`liveletters-diagnostics/src/reader.rs`](../../modules/liveletters-diagnostics/src/reader.rs) — `DiagnosticsReader::build_snapshot`.
- [`liveletters-diagnostics/src/dto.rs`](../../modules/liveletters-diagnostics/src/dto.rs) — `SyncHealth` и его геттеры.
- [`liveletters-store/INTERFACE.md`](../../modules/liveletters-store/INTERFACE.md) — `list_deferred_events(limit)`, `table_size(name)`, `InvalidTable`.
