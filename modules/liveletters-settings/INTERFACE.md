# `liveletters-settings` INTERFACE

## Назначение

`liveletters-settings` — библиотечный крейт, реализующий команду
`lltt settings`. Показывает и изменяет настройки трёх слоёв:

- `user_settings` и `mail_settings` в SQLite (SMTP/IMAP-параметры, никнейм, адрес почты);
- `GlobalConfig` в TOML (`config.toml`), в первую очередь секция `[log]` (включение/выключение журнала, уровень, размер файла, число архивов, разрешение на запись тел писем).

Идентичностью (`identities/<name>.toml`) управляет `lltt cu` — это другой слой; `settings` с ним не пересекается.

## Где находится интерфейс

- crate: `liveletters-settings`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- `Args` — clap-аргументы (`tokens: Vec<String>` для ручного разбора);
- `SettingsAction` — enum `{ Show, Set { key, value } }`;
- `SettingsError` — типизированные ошибки команды (включая `InvalidLogValue`);
- `run(ctx, args) -> Result<(), Box<dyn Error + Send + Sync>>` — единая точка запуска;
- `print_settings(...)` — печать;
- `CommandContext` (реэкспорт из `liveletters-output`);
- константы `COMMAND_NAME`, `summary()`, `crate_name()`.

## Сигнатура запуска

```rust
pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>>
```

## Подкоманды

### `show` (по умолчанию)

```
lltt settings
lltt settings show
```

Печатает `print_kv`-сводки по таблицам БД и, если хотя бы одно из полей `[log]` отличается от дефолта, отдельную секцию `[логирование]`.

```
[user_settings]
profile_id:        <id>
nickname:          <text>
email_address:     <адрес>
avatar_url:        <url>
language:          <ru|en>
setup_completed:   <true|false>

[mail_settings]
smtp.host:         <хост>
smtp.port:         <порт>
smtp.security:     <tls|starttls|none>
smtp.username:     <имя>
smtp.password:     <********>     (маскируется; --reveal отсутствует — пароли в текущей версии не показываются)
smtp.hello_domain: <домен>
imap.host:         <хост>
imap.port:         <порт>
imap.security:     <tls|starttls|none>
imap.username:     <имя>
imap.password:     <********>
imap.mailbox:      <папка>
imap.initial_lookback_days: <целое неотрицательное>

[логирование]
destination:       <file|stderr|none>
level:             <off|error|warn|info|debug|trace>
max_size_bytes:    <число>
keep_files:        <число>
include_bodies:    <true|false>
```

Секция `[логирование]` печатается только если хотя бы одно поле `log.*` отличается от дефолта. Если в `config.toml` ничего не задано — пользователь видит только настройки из БД, без упоминания журнала.

Если в БД нет записей — печатается строка `[user_settings] отсутствует` с подсказкой запустить `lltt settings set ...`.

### `set`

```
lltt settings set <ключ> <значение>
```

Допустимые ключи (23 штуки, жёсткий список в `set::ALLOWED_KEYS`):

- `nickname` — `user_settings.nickname`;
- `email_address` — `user_settings.email_address`;
- `avatar_url` — `user_settings.avatar_url` (пустая строка → `NULL`);
- `language` — `user_settings.language` (`ru` или `en`; иное значение отвергается `SettingsError::InvalidValue`);
- `setup_completed` — `user_settings.setup_completed` (`true` / `1` → 1, иначе 0);
- `smtp.host`, `smtp.port`, `smtp.security`, `smtp.username`,
  `smtp.password`, `smtp.hello_domain` — `mail_settings.smtp_*`;
- `imap.host`, `imap.port`, `imap.security`, `imap.username`,
  `imap.password`, `imap.mailbox` — `mail_settings.imap_*`;
- `imap.initial_lookback_days` — `mail_settings.initial_lookback_days`
  (целое неотрицательное; `0` — с самого начала, `1` — по умолчанию;
  нечисловое или отрицательное значение отвергается
  `SettingsError::InvalidValue`; применяется только при самом
  первом sync, пока в `sync_cursors` нет записи);
- `log.destination`, `log.level`, `log.max_size_bytes`, `log.keep_files`, `log.include_bodies` — `GlobalConfig.log` (TOML `config.toml`).

Пароли (`smtp.password`, `imap.password`) проходят через
`SecretBox::obfuscate` и в БД хранятся в формате `obf:v1:…`. При чтении
через `get_mail_settings_record` пароль автоматически расшифровывается
(с ленивой миграцией старого plaintext в обфусцированную форму).

Поля `log.*` валидируются через `LogConfig::set_field` и при ошибке
дают `SettingsError::InvalidLogValue(String)`. Допустимые значения
`log.level`: `off` / `error` / `warn` / `info` / `debug` / `trace`
(алиасы: `none`/`disabled`, `err`, `warning`).
Допустимые значения `log.destination`: `file` / `stderr` / `none`
(`off`/`disabled` — алиасы `none`).
`log.max_size_bytes` и `log.keep_files` — целые числа (`0` означает
«использовать дефолт»).
`log.include_bodies` — `true`/`false` (`1`/`0`, `yes`/`no`, `да`/`нет`).

При первом `set` запись в соответствующей таблице БД создаётся с
дефолтными значениями (через `ensure_records_exist`): для `user_settings`
язык `language` берётся из `liveletters_i18n::detect_system_locale()`,
то есть из переменных окружения `LC_ALL`/`LC_MESSAGES`/`LANG`
(поддерживается `ru` или `en`, иначе `en`). Затем перезаписывается одно
поле. Для `log.*` запись создаётся/обновляется в `config.toml` целиком
(`load_global` → мутация → `save_global`).

## `SettingsError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),
    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),
    #[error("неизвестный ключ: {0}")]
    InvalidKey(String),
    #[error("неверные аргументы: {0}")]
    InvalidArgs(String),
    #[error("некорректное значение настройки: {field}: {message}")]
    InvalidValue { field: String, message: String },
    #[error("некорректное значение журнала: {0}")]
    InvalidLogValue(String),
}
```

## Текущее состояние

Реализация готова. `show` читает три источника (БД, `[log]`, текущий пользователь) и печатает; `set` валидирует ключ, маршрутизирует в БД или `config.toml` и обновляет одно поле (с обфускацией паролей).

## Связанные документы

- [`liveletters-store/INTERFACE.md`](../../modules/liveletters-store/INTERFACE.md) — `get_user_settings_record`, `get_mail_settings_record`, `update_user_settings_field`, `update_mail_settings_field`.
- [`liveletters-i18n/INTERFACE.md`](../../modules/liveletters-i18n/INTERFACE.md) — `parse_locale` (используется для валидации `language`).
- [`liveletters-config/INTERFACE.md`](../../modules/liveletters-config/INTERFACE.md) — `load_global`, `save_global`, `GlobalConfig.log`.
- [`liveletters-log/INTERFACE.md`](../../modules/liveletters-log/INTERFACE.md) — `LogConfig::set_field`, `LogLevel`, `LogDestination`.
- [`liveletters-output/INTERFACE.md`](../../modules/liveletters-output/INTERFACE.md) — `print_kv`, `mask_password`.
- [`liveletters-cu/INTERFACE.md`](../../modules/liveletters-cu/INTERFACE.md) — управление идентичностью (другой слой).
