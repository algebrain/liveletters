# liveletters-config

## Назначение

`liveletters-config` это библиотека TOML-конфигурации LiveLetters. Она отвечает за парсинг, сериализацию и ввод-вывод конфигурации идентичностей, разрешение имени активной идентичности, а также за мост между дисковой формой `IdentityConfig` и оперативной формой `AppSettings`.

## Зона ответственности

- парсинг и сериализация `IdentityConfig` / `IdentityMeta` / `MailSettings` / `SmtpSettings` / `ImapSettings` / `ResourceSubscription` / `GlobalConfig`;
- ввод-вывод на диск: `home/config.toml`, `home/current-user` и `home/identities/<name>.toml`;
- чтение и запись имени текущего пользователя liveletters через `read_current_identity` / `write_current_identity`;
- мост `IdentityConfig` ↔ `AppSettings`;
- типизированные ошибки `ConfigError` для IO, TOML-парсинга, отсутствующей идентичности и отсутствующего `current-user`.

## Что модуль не должен делать

- валидировать содержимое email-адресов;
- подтверждать и скрывать пароли; TOML только хранит `password` и флаг `pwd_obfuscate`, а скрытие выполняет `lltt user add` через `liveletters-secret-box`;
- открывать сетевые соединения;
- реализовывать SMTP/IMAP;
- принимать имя текущего пользователя из CLI-флагов или переменных окружения (переменная `LLTT_CU` явно не поддерживается);
- делать миграции `schema_version` (зарезервировано, но не реализовано).

## Раскладка файлов в home-каталоге

```
<home>/
├── config.toml              ← load_global (read-only в текущей версии)
├── current-user             ← read_current_identity / write_current_identity
└── identities/
    ├── alice.toml
    └── bob.toml
```

- `config.toml` содержит только `schema_version: u32`. При отсутствии файла `load_global` возвращает `GlobalConfig::default()` с `schema_version = 1`.
- `current-user` — текстовый файл, одна строка с именем текущего пользователя liveletters. При отсутствии файла `read_current_identity` возвращает `ConfigError::NoCurrentUser(path)`.
- `identities/<name>.toml` — файл на идентичность. `save_identity` создаёт каталог `identities/` через `fs::create_dir_all` при первом сохранении.
- `load_identity` возвращает `ConfigError::UnknownIdentity(name)`, если файл не существует.
- `list_identities` возвращает отсортированный список имён без расширения `.toml`; при отсутствии каталога `identities/` возвращается пустой `Vec`.

## Почему выделен в отдельный крейт

`liveletters-config` сознательно отделён от `liveletters-store` и `liveletters-app-core` по двум причинам.

**Другая схема обновления.** Конфигурация живёт в TOML-файлах, которые редактируются пользователем или CLI-командами `lltt user init` / `lltt user add` / `lltt cu`. Хранилище — в SQLite, обновляется через `Store::*`-методы. Смешивать эти два контура в одном крейте означало бы смешивать разные модели конкурентного доступа, разные схемы ошибок и разные способы миграции.

**Другая семантика данных.** `IdentityConfig` описывает «внешние» параметры LiveLetters-инстанции: адреса почтовых серверов, аккаунты, ресурсы. `Store` оперирует «внутренним» материализованным состоянием: посты, комментарии, raw-журналы. Эти два класса данных изменяются по разным причинам, в разное время и разными людьми (администратор vs. пользователь).

При этом крейт не пытается быть «общим конфиг-фреймворком»: он не валидирует схемы, не строит dependency graph, не поддерживает несколько профилей окружения.

## Подтаблица `[meta]`: workaround под `toml` 0.8

В `IdentityConfig` поля `resources_owned` и `subscriptions` обёрнуты в `IdentityMeta` и сериализуются в подтаблицу `[meta]`. Это не стилистический выбор, а workaround под особенность `toml` 0.8: десериализатор `toml::from_str` молча отбрасывает `Vec<_>` на верхнем уровне после того, как в файле появился заголовок `[table]`.

В нашем случае порядок полей такой: `display_name` — это плоская строка на верхнем уровне, `[mail]` — первая подтаблица. Если бы `resources_owned` и `subscriptions` остались плоскими на верхнем уровне, `toml::from_str` их бы проглотил молча, без сообщения об ошибке, и они бы десериализовались в пустые `Vec`.

Обёртывание в `[meta]` решает эту проблему ценой чуть менее «плоского» TOML. С точки зрения пользователя крейта это прозрачно: методы `identity.resources_owned()` и `identity.subscriptions()` работают с подтаблицей внутри.

Этот workaround задокументирован в `tests/parse.rs::parses_full_identity_toml_with_subscriptions` как «минимальный TOML, который проходит парсинг с подписками».

## `MailSecurity`: lowercase и forward-compat

`MailSecurity` сериализуется в lowercase:

- `MailSecurity::None` ↔ `"none"`;
- `MailSecurity::StartTls` ↔ `"starttls"`;
- `MailSecurity::Tls` ↔ `"tls"`.

Метод `as_str(&self) -> &'static str` возвращает то же строковое представление, что и serde. Это позволяет конвертировать `MailSecurity` в `String` без serde-стека и без рассинхрона между serde-выводом и обычным `Display`.

Десериализация принимает дополнительные пользовательские синонимы `"ssl"`, `"SSL"`, `"ssl/tls"` как `MailSecurity::Tls`. Это нужно потому, что документация почтовых серверов часто называет режим TLS на отдельном порту словом SSL.

При обратной конвертации из строки (`parse_mail_security` в `mapping.rs`) `"ssl"` и `"ssl/tls"` маппятся в `MailSecurity::Tls`, а любое другое неизвестное значение, включая пустую строку, маппится в `MailSecurity::StartTls`. Это сознательный forward-compat: если в `AppSettings` окажется новое значение вроде `"starttls-v2"`, парсер не уронит конфигурацию, а выберет самый совместимый режим.

## Имя текущего пользователя liveletters

`read_current_identity(home) -> Result<String, ConfigError>`:

1. если `<home>/current-user` существует — читает его, обрезает `\r\n` и пробелы, возвращает;
2. если файл отсутствует — возвращает `Err(ConfigError::NoCurrentUser(home.join("current-user")))`. Это нормальное состояние сразу после `lltt init`, пока пользователь не выбран через `lltt cu <имя>`.

`write_current_identity(home, name) -> Result<(), ConfigError>`:

1. пишет `name` в `<home>/current-user` (одна запись `fs::write`, без `\n` на конце — `trim()` на чтении компенсирует);
2. если родительский каталог `home` не существует — `fs::write` вернёт `io::Error`, которое конвертируется в `ConfigError::Io(_)` через `From`.

`current_user_path(home) -> PathBuf`:

- `home.join("current-user")` — для диагностических сообщений и `assert`-тестов.

Файл `current-user` — **единственный источник истины** о том, кто сейчас выбран текущим пользователем. Переменная окружения `LLTT_CU` и CLI-флаг `--as` сейчас **не поддерживаются** (см. [apps/lltt/TECHNICAL_SPEC.md](../../apps/lltt/TECHNICAL_SPEC.md) — отложено на будущее). Это сознательное решение: иначе состояние «выбран через `lltt cu`, но `LLTT_CU` указывает на другое» становилось бы неотлаживаемым.

Запись файла — ответственность команды `lltt cu <имя>` (переключение). Чтение — ответственность бинаря `apps/lltt` при построении `CommandContext` и команды `lltt cu`.

## Мост к `AppSettings`

`map_identity_to_settings` и `settings_to_identity` — единственные функции, которые тащат зависимость от `liveletters-app-core` в крейт. Это сознательная зависимость, и она односторонняя: `liveletters-app-core` ничего не знает про TOML-парсинг и не зависит от `liveletters-config`.

Маппинг усекающий:

- `map_identity_to_settings` не копирует `resources_owned` и `subscriptions`, потому что в `AppSettings` нет для них полей. Списки ресурсов и подписок читаются из `IdentityConfig` напрямую;
- `settings_to_identity` не восстанавливает `resources_owned` и `subscriptions` (использует `Default::default()`), потому что в `AppSettings` их нет.

Если редактировать `resources_owned` / `subscriptions` через `AppSettings` и потом прогонять через `settings_to_identity` + `save_identity`, данные будут потеряны. Это документированное ограничение, и CLI `lltt` работает с этими списками через `IdentityConfig` напрямую.

`mailbox` для `ImapSettings` по умолчанию равен `"INBOX"`. Это канонический дефолт RFC 3501 и одновременно — единственный разумный mailbox по умолчанию для IMAP-клиента. Дефолт задаётся через `#[serde(default = "default_mailbox")]`.

`password` в `SmtpSettings` и `ImapSettings` помечен `#[serde(default)]` — пустая строка означает «пароль не задан». `pwd_obfuscate` по умолчанию равен `true` и служит указанием для `lltt user add`: непустой открытый пароль нужно подтвердить скрытым вводом и заменить значением `obf:v1:...`. При переносе настроек в БД `liveletters-store` дополнительно сохраняет секреты в скрытом виде, если они ещё открытые.

## Формат ошибок

`ConfigError`:

- `Io(String)` — ошибка файловой системы. Содержит `to_string()` от `std::io::Error`.
- `Toml(String)` — ошибка `toml::from_str` / `toml::to_string_pretty`. Содержит сообщение парсера/сериализатора.
- `MissingField { field: &'static str }` — зарезервировано для будущей явной валидации. Сейчас не возвращается.
- `UnknownIdentity(String)` — `load_identity` не нашёл файл с указанным именем. Поле содержит имя идентичности, чтобы UI мог показать «не знаю идентичность `alice`».
- `NoCurrentUser(PathBuf)` — `read_current_identity` не нашёл файл `<home>/current-user`. Поле содержит путь к ожидаемому файлу, чтобы сообщение об ошибке могло его напечатать. Сообщение `Display` подсказывает создать и выбрать пользователя через `lltt user init`, `lltt user add` и `lltt cu <имя>`.

Реализован `From<std::io::Error>`, `From<toml::de::Error>`, `From<toml::ser::Error>`, поэтому `?` в вызывающем коде пробрасывает ошибки без обёртки.

`ConfigError` реализует `std::error::Error` вручную (без `#[derive(thiserror::Error)]`). Это сознательное упрощение: публичная поверхность крейта не зависит от `thiserror` как от макроса, и любая будущая замена `thiserror` на другой error-крейт не сломает API.

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- 5 src-файлов: `lib.rs`, `error.rs`, `global.rs`, `identity.rs`, `io.rs`, `mapping.rs`;
- lib-тест: `crate_name_is_set`;
- 6 тестов в `tests/parse.rs`: `parses_minimal_identity_toml`, `parses_full_identity_toml_with_subscriptions`, `parses_ssl_security_alias_as_tls`, `parse_rejects_missing_required_fields`, `parse_accepts_default_meta_when_omitted`, `pwd_obfuscate_defaults_to_true_when_omitted`;
- 6 тестов в `tests/io.rs`: `save_and_load_identity_round_trip`, `list_identities_returns_empty_when_dir_missing`, `list_identities_returns_all_saved_names_sorted`, `load_identity_returns_unknown_when_missing`, `load_global_returns_default_when_file_missing`, `identity_settings_round_trip_via_app_settings`;
- 1 smoke-тест в `tests/smoke.rs`: `crate_is_wired_into_workspace`;
- 4 теста в `tests/current_user.rs` (добавлены вместе с `read_current_identity` / `write_current_identity`): `read_current_identity_returns_no_current_user_when_file_missing`, `write_then_read_round_trip`, `write_current_identity_overwrites_previous_value`, `read_trims_trailing_newline_added_by_external_editor`;
- итого 17 тестов.

## Требования к структуре каталога

- `src/lib.rs`;
- `src/error.rs`;
- `src/global.rs`;
- `src/identity.rs`;
- `src/io.rs`;
- `src/mapping.rs`;
- `tests/parse.rs`;
- `tests/io.rs`;
- `tests/smoke.rs`.

Все src-файлы ≤ 176 строк. Лимит 600 строк соблюдается.

## Требования к тестам

Покрытие тестами обязательно.

Реализованные проверки:

- парсинг минимального `IdentityConfig` без SMTP/IMAP/meta;
- парсинг полного `IdentityConfig` со SMTP/IMAP/meta и двумя подписками;
- отказ `toml::from_str` на отсутствии обязательного `display_name`;
- дефолт `meta.resources_owned = []` и `meta.subscriptions = []`, если подтаблица `[meta]` опущена;
- round-trip `save_identity` → `load_identity` через `tempfile::TempDir`;
- `list_identities` возвращает пустой `Vec`, если каталог `identities/` отсутствует;
- `list_identities` возвращает имена в лексикографическом порядке;
- `load_identity` возвращает `ConfigError::UnknownIdentity(name)` на отсутствующий файл;
- `load_global` возвращает `GlobalConfig::default()` на отсутствующий `config.toml`;
- round-trip `IdentityConfig` → `AppSettings` → `IdentityConfig` сохраняет `display_name`, `mail.publish`, `SmtpSettings.{host, port, security}`, `ImapSettings.mailbox`;
- `crate_name` отдаёт правильное имя;
- 1 lib-тест `crate_name_is_set`;
- `read_current_identity` возвращает `ConfigError::NoCurrentUser` на отсутствующий файл;
- round-trip `write_current_identity` → `read_current_identity`;
- повторный `write_current_identity` перезаписывает предыдущее значение;
- `read_current_identity` обрезает завершающий `\n` (если файл редактировали руками).

## Требования к документации

Обязательна документация:

- описание раскладки файлов в home-каталоге;
- описание workaround подтаблицы `[meta]`;
- описание схемы `IdentityConfig` со всеми подтаблицами;
- описание `MailSecurity` и `as_str`;
- описание `read_current_identity` / `write_current_identity`;
- описание моста `map_identity_to_settings` / `settings_to_identity` и усечения `resources_owned` / `subscriptions`;
- описание вариантов `ConfigError`;
- явная фиксация того, что модуль не делает (подтверждение паролей, валидация email, миграции, выбор текущего пользователя).

## Критерии готовности

- `IdentityConfig` парсится из TOML-файла со всеми подтаблицами;
- `MailSecurity` сериализуется в lowercase и обратно;
- `save_identity` создаёт каталог `identities/` при первом сохранении;
- `load_identity` возвращает `ConfigError::UnknownIdentity` на отсутствующий файл;
- `load_global` возвращает `GlobalConfig::default()` на отсутствующий `config.toml`;
- `map_identity_to_settings` / `settings_to_identity` дают round-trip для базовых полей;
- 17 тестов зелёные.

Сейчас практически считаются уже закрытыми:

- выделение конфигурации в отдельный крейт;
- TOML-парсинг и сериализация;
- IO через `tempfile::TempDir` без in-memory fakes;
- мост к `AppSettings`;
- workaround под `toml` 0.8.

Модуль пока не считается завершенным в части:

- `save_global` для записи обновлений `schema_version`;
- миграции `schema_version` при изменении формата;
- явная валидация значений помимо serde-уровня;
- поддержка environment-variable подстановок в строках (например, `${SMTP_PASSWORD}`).

Эти направления зафиксированы как возможный следующий шаг, но не блокируют текущую версию.

## Связанные документы

- [idea.technical.md](../../docs/idea.technical.md)
- [technical-plan.md](../../docs/technical-plan.md)
- [liveletters-app-core INTERFACE.md](../liveletters-app-core/INTERFACE.md)
- [liveletters-store INTERFACE.md](../liveletters-store/INTERFACE.md)
