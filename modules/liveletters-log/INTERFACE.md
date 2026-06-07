# `liveletters-log` INTERFACE

## Назначение

`liveletters-log` это отдельный крейт, отвечающий за журнал сетевых операций и парсинга payload в LiveLetters. По умолчанию журнал **выключен**; включается одной командой `lltt settings set log.level info`.

Крейт сознательно отделён от `liveletters-mail` / `liveletters-sync` / `liveletters-lltt-sync` по двум причинам:

- общий логгер — это инфраструктурный крейт, у которого нет «бизнес-логики»; все потребители журнала зависят от него одинаково;
- тип `LogConfig` живёт в `liveletters-log`, а `liveletters-config` лишь ре-экспортирует его. Это разрывает цикл: `liveletters-config → liveletters-app-core`, и одновременно `liveletters-config → liveletters-log`, но `liveletters-log` ничего не знает про `liveletters-config`.

Что крейт делает:

- инициализирует глобальный логгер процесса (`init(&Path, &LogConfig)`);
- принимает сообщения пяти уровней (`log_error` / `log_warn` / `log_info` / `log_debug` / `log_trace`) и специальное `log_payload` для тел писем;
- ротирует файл по размеру, атомарно через `fs::rename`;
- завершает работу вызовом `shutdown` (сбрасывает буфер).

Что крейт **не** делает:

- не зависит от `tracing`, `log`, `env_logger` и любых других крейтов логирования (zero external deps, кроме `thiserror` для `LogError` и `serde` для `LogConfig`);
- не пишет в `stdout` / `syslog` / journald — только в файл, `stderr` или `none`;
- не раскрывает тела писем и payload, пока пользователь явно не поставит `log.include_bodies = true`;
- не предоставляет асинхронного API; все вызовы синхронные и блокирующие.

## Где находится интерфейс

- crate: `liveletters-log`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- типы `LogConfig`, `LogLevel`, `LogDestination` (ре-экспорт из собственного `config`; `liveletters-config` ре-экспортирует их же под тем же именем);
- функции инициализации: `init`, `shutdown`, `reset_for_tests`;
- функции доступа к активным параметрам: `max_size`, `keep_files`, `is_bodies_enabled`;
- функции записи: `log_error`, `log_warn`, `log_info`, `log_debug`, `log_trace`, `log_payload`;
- `LogError` (через `thiserror`).

Внутренние модули `level`, `writer`, `rotation` не публикуются; `init` экспортирован как `init` (а не `mod init`).

## Что считается внешним интерфейсом этого модуля

С практической точки зрения внешний интерфейс `liveletters-log` это:

1. тип `LogConfig` с методом `set_field(key, value)` для CLI-обновлений;
2. типы `LogLevel` и `LogDestination` (через `Display`, `FromStr` и `as_u8` / `as_str`);
3. функция `init(&Path, &LogConfig) -> Result<(), LogError>`;
4. шесть функций записи (`log_error`…`log_payload`);
5. функция `shutdown()` для сброса буфера.

Именно этим API пользуются `apps/lltt` (инициализация/завершение), `liveletters-mail` (IMAP/SMTP), `liveletters-sync` (ingest), `liveletters-lltt-sync` (pull/push) и `liveletters-settings` (запись `log.*` в `GlobalConfig`).

## Конфигурация: `LogConfig`

```rust
pub struct LogConfig {
    pub destination: LogDestination,   // по умолчанию File
    pub level: LogLevel,               // по умолчанию Off
    pub max_size_bytes: u64,           // по умолчанию 5 МиБ
    pub keep_files: u32,               // по умолчанию 3
    pub include_bodies: bool,          // по умолчанию false
}
```

`set_field(key: &str, value: &str) -> Result<(), String>` — единая точка обновления поля по строковому ключу (без префикса `log.`). Используется `liveletters-settings` при обработке `lltt settings set log.<key> <value>`.

### `LogLevel`

```rust
pub enum LogLevel {
    Off,    // 0
    Error,  // 1
    Warn,   // 2
    Info,   // 3
    Debug,  // 4
    Trace,  // 5
}
```

`as_u8()` — числовое значение для атомарного сравнения в hot-path.

`FromStr` принимает алиасы: `"off"` / `"none"` / `"disabled"`; `"error"` / `"err"`; `"warn"` / `"warning"`; `"info"`; `"debug"`; `"trace"`.

### `LogDestination`

```rust
pub enum LogDestination {
    File,    // <home>/logs/liveletters.log
    Stderr,  // стандартный поток ошибок
    None,    // ничего не пишется (sink-пустышка)
}
```

`FromStr` принимает `"file"` / `"stderr"` / `"none"` / `"off"` / `"disabled"`.

## Хранилище и ротация

Журнал пишется в `<home>/logs/liveletters.log`. Каталог создаётся `init` через `fs::create_dir_all`. При превышении `max_size_bytes` файл переименовывается в `liveletters.log.1`, старые `.1`/`.2`/… сдвигаются на `.2`/`.3`/…, и самый старый (`.N` при `keep_files = N`) удаляется. Атомарность — через `fs::rename`.

Минимумы, которые соблюдает `init` (и предупреждает в stderr при занижении):

- `max_size_bytes = 0` → используется `5 * 1024 * 1024`; значения ниже `1024` поднимаются до `1024`;
- `keep_files = 0` → используется `3`.

## Формат строк

```
2026-06-07T00:31:12.345Z INFO imap.connect host=imap.example.org port=993 security=tls
2026-06-07T00:31:12.789Z INFO sync.ingest outcome=applied message_id=imap-uid-42 event_id=…
2026-06-07T00:31:13.123Z ERROR smtp.connect error=connection refused
```

- `2026-06-07T00:31:12.345Z` — UTC ISO-8601 с миллисекундами;
- `INFO` / `ERROR` — уровень (`off`/`error`/`warn`/`info`/`debug`/`trace`);
- `imap.connect` — target (логический «подсистема.событие»; в текущей версии API target и message объединены в одну строку вида `target message` без точки с запятой);
- `host=…` `port=…` `error=…` — пары `key=value` (значения с пробелами экранируются кавычками).

## Поведение при выключенном журнале

Если `log.level = off` (по умолчанию), все шесть функций записи возвращают сразу после одной атомарной проверки `AtomicU8::load`. `init` всё равно открывает файл, чтобы переключение `lltt settings set log.level info` сразу начало писать. Если `destination = none`, запись всегда игнорируется.

## Ошибки: `LogError`

```rust
pub enum LogError {
    Io(std::io::Error),
}
```

`init` возвращает `Err` только при сбое `fs::create_dir_all` / `File::create`; неверные значения полей отсекаются на этапе `set_field` (возвращает `String` с описанием, не `LogError`).

## Примеры использования

### Инициализация в бинаре

```rust
use liveletters_config as config;
use liveletters_log;

fn main() -> std::process::ExitCode {
    let home = liveletters_store::resolve_data_dir_from_env()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let log = config::load_global(&home)
        .map(|cfg| cfg.log)
        .unwrap_or_default();
    if let Err(err) = liveletters_log::init(&home, &log) {
        eprintln!("предупреждение: не удалось инициализировать журнал: {err}");
    }

    // … основная работа …

    liveletters_log::shutdown();
    std::process::ExitCode::SUCCESS
}
```

### Запись из сетевого модуля

```rust
liveletters_log::log_info(format!(
    "imap.connect host={} port={} security={}",
    host, port, security,
));
liveletters_log::log_error(format!("smtp.connect error={error}"));
```

### Запись payload (только при `include_bodies = true`)

```rust
liveletters_log::log_payload(format!("imap.fetch message_id={message_id}"));
// игнорируется, пока `log.include_bodies = false`.
```

## Что модуль не делает

- не делает асинхронных вызовов (только синхронный `BufWriter` + `fs::write_all`);
- не пишет в `stdout` (только `stderr` или файл);
- не раскрывает пароли и токены `AUTH PLAIN` (это ответственность вызывающего);
- не подменяет `tracing` и `log` — это полностью самостоятельный логгер.
