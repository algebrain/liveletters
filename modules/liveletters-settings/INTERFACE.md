# `liveletters-settings` INTERFACE

## Назначение

`liveletters-settings` — библиотечный крейт, реализующий команду
`lltt settings`. Показывает и изменяет настройки, хранящиеся в таблицах
`user_settings` и `mail_settings` БД (SMTP/IMAP-параметры, никнейм,
адрес почты). Идентичностью (`identities/<name>.toml`) управляет
`lltt cu` — это другой слой; `settings` с ним не пересекается.

## Где находится интерфейс

- crate: `liveletters-settings`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- `Args` — clap-аргументы (`tokens: Vec<String>` для ручного разбора);
- `SettingsAction` — enum `{ Show, Set { key, value } }`;
- `SettingsError` — типизированные ошибки команды;
- `run(ctx, args) -> Result<(), Box<dyn Error + Send + Sync>>` — единая точка запуска;
- `print_settings(&Option<UserSettingsRecord>, &Option<MailSettingsRecord>)` — печать;
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

Печатает `print_kv`-сводки по двум таблицам:

```
[user_settings]
profile_id:        <id>
nickname:          <text>
email_address:     <адрес>
avatar_url:        <url>
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
```

Если записи в БД нет — печатается строка `[user_settings] отсутствует`
с подсказкой запустить `lltt settings set ...`.

### `set`

```
lltt settings set <ключ> <значение>
```

Допустимые ключи (16 штук, жёсткий список в `set::ALLOWED_KEYS`):

- `nickname` — `user_settings.nickname`;
- `email_address` — `user_settings.email_address`;
- `avatar_url` — `user_settings.avatar_url` (пустая строка → `NULL`);
- `setup_completed` — `user_settings.setup_completed` (`true` / `1` → 1, иначе 0);
- `smtp.host`, `smtp.port`, `smtp.security`, `smtp.username`,
  `smtp.password`, `smtp.hello_domain` — `mail_settings.smtp_*`;
- `imap.host`, `imap.port`, `imap.security`, `imap.username`,
  `imap.password`, `imap.mailbox` — `mail_settings.imap_*`.

Пароли (`smtp.password`, `imap.password`) проходят через
`SecretBox::obfuscate` и в БД хранятся в формате `obf:v1:…`. При чтении
через `get_mail_settings_record` пароль автоматически расшифровывается
(с ленивой миграцией старого plaintext в обфусцированную форму).

При первом `set` запись в соответствующей таблице создаётся с
дефолтными значениями (через `ensure_records_exist`), затем
перезаписывается одно поле.

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
}
```

## Текущее состояние

Реализация готова. `show` читает две таблицы и печатает; `set`
валидирует ключ и обновляет одно поле (с обфускацией паролей).

## Связанные документы

- [`liveletters-store/INTERFACE.md`](../../modules/liveletters-store/INTERFACE.md) — `get_user_settings_record`, `get_mail_settings_record`, `update_user_settings_field`, `update_mail_settings_field`.
- [`liveletters-output/INTERFACE.md`](../../modules/liveletters-output/INTERFACE.md) — `print_kv`, `mask_password`.
- [`liveletters-cu/INTERFACE.md`](../../modules/liveletters-cu/INTERFACE.md) — управление идентичностью (другой слой).
