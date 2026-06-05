# liveletters-settings

## Назначение

`liveletters-settings` — командный крейт `lltt settings`. Показывает и
изменяет настройки, хранящиеся в таблицах `user_settings` и
`mail_settings` БД. Идентичностью (`identities/<name>.toml`) управляет
`lltt cu` — это другой слой; `settings` с ним не пересекается.

## Зона ответственности

Крейт отвечает за:

- открытие `Store` через `Store::open_for_home_dir(&ctx.home)`;
- чтение/запись в таблицах `user_settings` / `mail_settings`;
- маскирование паролей через `SecretBox::obfuscate` (для записи) и
  `mask_password` (для печати);
- валидацию ключа `set` через жёсткий список `ALLOWED_KEYS` (16 значений);
- парсинг первого аргумента как `SettingsAction::{Show, Set}`.

Крейт **не** отвечает за:

- управление идентичностью (`identities/<name>.toml`) — это `lltt cu`;
- хранение паролей в открытом виде — `obf:v1:…` обязателен;
- сетевые подключения — `settings` только читает/пишет БД.

## Текущее состояние реализации

- `Args { tokens: Vec<String> }` — позиционные токены, разбираются вручную;
- `SettingsAction::{Show, Set { key, value }}` — внутреннее представление;
- `SettingsError { Store, Config, InvalidKey, InvalidArgs }` — `thiserror`;
- `print_settings(user, mail)` — печатает 5 + 12 строк через `print_kv`;
- `show::run(home, identity_name)` — чтение из двух таблиц и печать;
- `set::run(home, identity_name, key, value)` — валидация + `ensure_records_exist` + `update_*_settings_field`;
- 9 интеграционных тестов.

## Критерии готовности

- `cargo build -p liveletters-settings` зелёный;
- `cargo test -p liveletters-settings` зелёный;
- `lltt settings` после `lltt init` печатает `[user_settings] отсутствует`;
- `lltt settings set smtp.host imap.example.org` сохраняет значение в БД;
- `lltt settings set smtp.password secret` сохраняет обфусцированную форму `obf:v1:…`;
- `lltt settings set bogus.key value` возвращает ошибку `InvalidKey`.

## Связанные документы

- [`liveletters-store/src/settings.rs`](../../modules/liveletters-store/src/settings.rs) — `update_user_settings_field`, `update_mail_settings_field`, `obfuscate_secret_if_needed`, `reveal_secret_with_lazy_migration`.
- [`liveletters-store/src/secret_bridge.rs`](../../modules/liveletters-store/src/secret_bridge.rs) — `secret_bridge::obfuscate` / `deobfuscate`.
- [`liveletters-cu/INTERFACE.md`](../../modules/liveletters-cu/INTERFACE.md) — граница слоя «идентичность».
