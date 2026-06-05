# `liveletters-doctor` INTERFACE

## Назначение

`liveletters-doctor` — библиотечный крейт, реализующий команду
`lltt doctor`. Команда печатает сводку состояния синхронизации:
здоровье системы, число писем в каждой категории (`Applied | Duplicate |
Replay | Unauthorized | Invalid | Malformed | Deferred`), число
неотправленных событий в outbox. С флагом `--verbose` дополнительно
печатает три секции: deferred-события, identities и размеры таблиц БД.

## Где находится интерфейс

- crate: `liveletters-doctor`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- `Args { verbose: bool }` — clap-аргументы команды (флаги `--verbose` / `-v`);
- `DoctorError` — типизированные ошибки команды;
- `run(ctx, args) -> Result<(), Box<dyn Error + Send + Sync>>` — единая точка запуска;
- `print_doctor(&DiagnosticsSnapshot)` — печать 9 стандартных строк;
- `print_doctor_verbose(&DiagnosticsSnapshot, &Store, &Path)` — расширенная печать;
- `CommandContext` (реэкспорт из `liveletters-output`);
- константы `COMMAND_NAME`, `summary()`, `crate_name()`.

## Сигнатура запуска

```rust
#[derive(Debug, Default, clap::Args)]
pub struct Args {
    /// Расширенный вывод: deferred-события, identities, размер таблиц.
    #[arg(long, short = 'v')]
    pub verbose: bool,
}

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>>
```

## Формат вывода

### Без `--verbose` — 9 строк формата `key: value`

```
здоровье: <здоров | деградирован>
Applied: <usize>
Duplicate: <usize>
Replay: <usize>
Unauthorized: <usize>
Invalid: <usize>
Malformed: <usize>
Deferred: <usize>
Outbox (исходящих): <usize>
```

`здоровье` = `Healthy`, если `malformed_messages + unauthorized_messages +
invalid_messages + deferred_events == 0`; иначе `Degraded`.

### С `--verbose` — стандартные 9 строк + 3 секции

```
<9 строк выше>

--- deferred_events (последние 10) ---
  - <event_id>: <reason>
  ...
(или "(нет)")

--- identities ---
  <N> конфигов: <список имён>; текущий: <current-user>
(или "(каталог identities/ отсутствует)")

--- таблицы ---
  posts: <bytes> B
  comments: <bytes> B
  outbox: <bytes> B
  raw_messages: <bytes> B
  deferred_events: <bytes> B
  subscriptions: <bytes> B
```

Размеры таблиц берутся из SQLite-таблицы `dbstat`
(`SUM(pgsize) WHERE name = ?`); `store.table_size(name)` — обёртка
с белым списком имён таблиц (`InvalidTable` при попытке подсунуть
произвольную строку).

## Текущее состояние

Реализация готова. Источник данных — существующий
`liveletters_diagnostics::DiagnosticsReader::build_snapshot` плюс
`Store::list_deferred_events(limit)` и `Store::table_size(name)`.

## Связанные документы

- [`liveletters-diagnostics/INTERFACE.md`](../../modules/liveletters-diagnostics/INTERFACE.md) — `DiagnosticsSnapshot`, `SyncHealth`, `HealthStatus`.
- [`liveletters-output/INTERFACE.md`](../../modules/liveletters-output/INTERFACE.md) — `print_kv`.
- [`liveletters-store/INTERFACE.md`](../../modules/liveletters-store/INTERFACE.md) — `list_deferred_events`, `table_size`, `InvalidTable`.
