# `liveletters-config` INTERFACE

## Назначение

`liveletters-config` это отдельный крейт, отвечающий за всё, что связано с TOML-конфигурацией LiveLetters на диске: парсинг `IdentityConfig` из файла, сериализация обратно, загрузка глобальной конфигурации, перечисление и разрешение имён идентичностей, а также мост между `IdentityConfig` (диск) и `AppSettings` (память, крейт `liveletters-app-core`).

Крейт сознательно отделён от `liveletters-store` и `liveletters-app-core` по двум причинам:

- конфигурация живёт в TOML-файлах, а не в SQLite, и имеет другую схему обновления (ручное редактирование, `lltt user add`, миграции через `schema_version` в `GlobalConfig`);
- конфигурация описывает «внешние» параметры LiveLetters-инстанции (адреса почтовых серверов, аккаунты, ресурсы), а не «внутреннее» материализованное состояние.

Что крейт делает:

- читает/пишет `home/config.toml` (глобальные настройки) и `home/identities/<name>.toml` (настройки конкретной идентичности);
- типизирует формы `IdentityConfig`, `IdentityMeta`, `MailSettings`, `SmtpSettings`, `ImapSettings`, `MailSecurity`, `ResourceSubscription`, `GlobalConfig`;
- читает и записывает имя текущего пользователя liveletters в файл `home/current-user`;
- конвертирует `IdentityConfig` ↔ `AppSettings` для интеграции с runtime-слоем;
- ре-экспортирует `LogConfig`, `LogLevel`, `LogDestination` из крейта `liveletters-log` для пользователей, которым нужно единое «конфигурационное» импортирование.

Что крейт **не** делает:

- не подтверждает пароли через терминал и не открывает хранилище секретов; TOML может содержать как открытые строки, так и уже скрытые значения `obf:v1:...`, а решение о скрытии принимает команда `lltt user add` по полю `pwd_obfuscate`;
- не открывает сетевые соединения;
- не запускает SMTP/IMAP и не читает почту;
- не принимает решений о том, какое имя «активно», из CLI-флагов или переменных окружения — имя текущего пользователя liveletters берётся ТОЛЬКО из файла `home/current-user` (см. CLI `lltt cu`).

## Где находится интерфейс

- crate: `liveletters-config`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- структуры `IdentityConfig`, `IdentityMeta`, `MailSettings`, `SmtpSettings`, `ImapSettings`, `ResourceSubscription`, `GlobalConfig`;
- перечисление `MailSecurity` с методами `as_str(&self) -> &'static str`;
- `ConfigError` (включая вариант `NoCurrentUser(PathBuf)`);
- функции ввода-вывода: `load_global`, `load_identity`, `save_identity`, `save_global`, `list_identities`;
- функции работы с текущим пользователем liveletters: `read_current_identity`, `write_current_identity`, `current_user_path`;
- функции моста: `map_identity_to_settings`, `settings_to_identity`;
- ре-экспорт `LogConfig`, `LogLevel`, `LogDestination` из `liveletters-log` (для удобства импорта из `liveletters-config`);
- функция `crate_name() -> &'static str` для диагностики.

Внутренние модули `error`, `global`, `identity`, `io`, `mapping` не публикуются.

## Что считается внешним интерфейсом этого модуля

С практической точки зрения внешний интерфейс `liveletters-config` это:

1. формы `IdentityConfig` / `IdentityMeta` / `MailSettings` / `SmtpSettings` / `ImapSettings` / `ResourceSubscription` для парсинга/сериализации TOML;
2. `MailSecurity` с гарантированной сериализацией в lowercase;
3. пять функций ввода-вывода: `load_global`, `load_identity`, `save_identity`, `save_global`, `list_identities`;
4. три функции работы с текущим пользователем: `read_current_identity`, `write_current_identity`, `current_user_path`;
5. две функции моста к `AppSettings`: `map_identity_to_settings` / `settings_to_identity`;
6. `ConfigError` как единый тип ошибок конфигурации (включая вариант `NoCurrentUser`);
7. `LogConfig` / `LogLevel` / `LogDestination` (ре-экспорт из `liveletters-log`).

Именно этим API пользуется CLI `apps/lltt` (команды `lltt user …`, `lltt cu …`, `lltt init`, `lltt settings …`) и integration-тесты `tests/parse.rs` / `tests/io.rs` / `tests/smoke.rs` / `tests/current_user.rs`.

## Раскладка файлов в home-каталоге

```
<home>/
├── config.toml              ← load_global / save_global
├── logs/
│   └── liveletters.log      ← журнал, см. liveletters-log
├── current-user             ← read_current_identity / write_current_identity
├── identities/
│   ├── alice.toml           ← save_identity(home, "alice", …)
│   └── bob.toml             ← save_identity(home, "bob", …)
└── users/
    └── <name>/
        └── config.toml      ← SecurityConfig (per-user настройки безопасности)
```

- `config.toml` содержит `schema_version: u32` и опциональную секцию `[log]`; при отсутствии файла `load_global` возвращает `GlobalConfig::default()`;
- `current-user` — текстовый файл, одна строка, имя текущего пользователя liveletters без расширения; при отсутствии файла `read_current_identity` возвращает `ConfigError::NoCurrentUser(path)`;
- каждая идентичность — отдельный файл в `identities/`; имя файла — это имя идентичности плюс расширение `.toml`;
- `save_identity` создаёт каталог `identities/` через `fs::create_dir_all` при первом сохранении;
- `load_identity` возвращает `ConfigError::UnknownIdentity(name)`, если файл не существует;
- `list_identities` возвращает отсортированный список имён без расширения `.toml`; если каталог `identities/` отсутствует, возвращается пустой `Vec`;
- `users/<name>/config.toml` — per-user настройки безопасности (`SecurityConfig`); создаётся один раз при `lltt user add`, читается командным слоем sync/инбокса; подробно — в разделе «`SecurityConfig`» ниже.

## Структура `IdentityConfig`

```rust
pub struct IdentityConfig {
    pub display_name: String,
    pub mail: MailSettings,
    #[serde(default)]
    pub meta: IdentityMeta,
}
```

Минимальный валидный TOML-файл:

```toml
display_name = "Alice"

[mail]
publish = "alice-publish@example.org"
receive = ["alice-feed@example.org"]
```

Полный TOML-файл с SMTP/IMAP/метаданными:

```toml
display_name = "Bob"

[mail]
publish = "bob-publish@example.org"
receive = ["bob-feed@example.org", "bob-feed2@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "bob"
password = ""
pwd_obfuscate = true
hello_domain = "example.org"

[mail.imap]
host = "imap.example.org"
port = 143
security = "tls"
username = "bob"
password = ""
pwd_obfuscate = true
mailbox = "INBOX"

[meta]
resources_owned = ["blog-1", "blog-2"]

[[meta.subscriptions]]
resource_id = "alice-blog"
delivery_address = "bob-feed@example.org"

[[meta.subscriptions]]
resource_id = "carol-blog"
delivery_address = "bob-feed2@example.org"
```

`display_name` — обязательный, непустой никнейм, отображаемый в UI.

`mail` — обязательная вложенная таблица с настройками почты (см. ниже).

`meta` — необязательная (serde default) вложенная таблица со списками ресурсов и подписок.

### Подтаблица `[meta]`: зачем она

`meta` намеренно выделена в отдельный подраздел TOML, а не лежит «в плоскости» рядом с `display_name`. Это workaround под особенность `toml` 0.8: десериализатор `toml::from_str` молча отбрасывает `Vec<_>` на верхнем уровне после заголовка `[table]`. После того как в файле появилась первая подтаблица (`[mail]`), верхнеуровневый `Vec` теряется.

Обёртывание `resources_owned` и `subscriptions` в `[meta]` решает эту проблему ценой чуть менее «плоского» TOML. С точки зрения API это прозрачно: пользователь крейта работает с `identity.resources_owned()` и `identity.subscriptions()`, не задумываясь о подтаблице.

### `IdentityMeta`

```rust
pub struct IdentityMeta {
    #[serde(default)]
    pub resources_owned: Vec<String>,
    #[serde(default)]
    pub subscriptions: Vec<ResourceSubscription>,
}
```

- `resources_owned` — список `resource_id`, владельцем которых является эта идентичность (например, `["blog-1", "blog-2"]`).
- `subscriptions` — список подписок, каждая из которых содержит `resource_id` и `delivery_address` (адрес для входящей почты, на который подписан пользователь).

### `MailSettings`

```rust
pub struct MailSettings {
    pub publish: String,
    #[serde(default)]
    pub receive: Vec<String>,
    #[serde(default)]
    pub smtp: Option<SmtpSettings>,
    #[serde(default)]
    pub imap: Option<ImapSettings>,
}
```

- `publish` — обязательный адрес, с которого идентичность публикует посты.
- `receive` — необязательный (serde default) список адресов, на которые приходят входящие письма.
- `smtp` — необязательный блок настроек исходящей почты.
- `imap` — необязательный блок настроек входящей почты.

Отсутствие `smtp` или `imap` означает «не настроено», а не «дефолт». Это позволяет иметь идентичности только для чтения (без `smtp`) или только для записи (без `imap`).

### `SmtpSettings` / `ImapSettings`

```rust
pub struct SmtpSettings {
    pub host: String,
    pub port: u16,
    pub security: MailSecurity,
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_pwd_obfuscate")]
    pub pwd_obfuscate: bool,
    #[serde(default)]
    pub hello_domain: String,
}

pub struct ImapSettings {
    pub host: String,
    pub port: u16,
    pub security: MailSecurity,
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_pwd_obfuscate")]
    pub pwd_obfuscate: bool,
    #[serde(default = "default_mailbox")]
    pub mailbox: String,
}
```

`mailbox` в `ImapSettings` по умолчанию равен `"INBOX"`. Это не `Option<String>` сознательно: для IMAP всегда нужен какой-то ящик, и `"INBOX"` — канонический дефолт RFC 3501.

`password` и в `SmtpSettings`, и в `ImapSettings` помечен `#[serde(default)]` — это позволяет иметь конфиг без пароля (например, для server-side auth или для последующего ввода через CLI). Пустая строка означает «пароль не задан».

`pwd_obfuscate` по умолчанию равен `true`. Это не само скрытие пароля, а указание для `lltt user add`: если пароль непустой и ещё не начинается с `obf:v1:`, команда попросит подтвердить его скрытым вводом и заменит открытое значение на скрытое.

`hello_domain` в `SmtpSettings` по умолчанию пустой; транспортный слой может вывести его из адреса отправителя, если явное значение не задано.

### `MailSecurity`

```rust
#[serde(rename_all = "lowercase")]
pub enum MailSecurity {
    None,
    StartTls,
    Tls,
}
```

Сериализуется в lowercase: `"none"`, `"starttls"`, `"tls"`. При чтении TOML дополнительно принимает `"ssl"`, `"SSL"` и `"ssl/tls"` как синонимы `"tls"`. `as_str(&self) -> &'static str` возвращает каноническое строковое представление, чтобы можно было конвертировать `MailSecurity` в `String` без `serde_json`.

### `ResourceSubscription`

```rust
pub struct ResourceSubscription {
    pub resource_id: String,
    pub delivery_address: String,
}

impl ResourceSubscription {
    pub fn new(resource_id: impl Into<String>, delivery_address: impl Into<String>) -> Self
}
```

`new(...)` — единственный «удобный» конструктор, остальное — через публичные поля.

## Глобальная конфигурация: `GlobalConfig`

```rust
pub struct GlobalConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub log: LogConfig,
}
```

`schema_version: u32` со значением по умолчанию `1` — зарезервировано для будущих миграций формата.

`log: LogConfig` — настройки журнала (см. ниже). При отсутствии секции `[log]` в TOML используется `LogConfig::default()`.

`load_global(home)`:

- если `home/config.toml` отсутствует, возвращает `GlobalConfig::default()` без ошибки;
- иначе парсит TOML и возвращает полученный `GlobalConfig`;
- при ошибке парсинга возвращает `ConfigError::Toml(_)`.

`save_global(home, &GlobalConfig)` — атомарно записывает глобальный конфиг в `home/config.toml`; используется командой `lltt settings set log.*`.

## Per-user настройки безопасности: `SecurityConfig`

`SecurityConfig` объединяет все квоты и лимиты, ранее жившие как кодовые
константы: `mime_limits` (из `liveletters-mime`), `ingest_limits` и
`retention` (из `liveletters-sync`). Файл — `users/<name>/config.toml`;
создаётся один раз при `lltt user add` и читается командным слоем sync/инбокса.
Файл намеренно не документирован и не редактируется через `lltt settings`;
ручные правки возможны на свой страх и риск.

Поведение при чтении — «переопределение»: каждое поле подсекции несёт
собственный serde-default. Отсутствующий в файле ключ заменяется кодовым
значением; заданный — уважается. Пример частичного переопределения:

```toml
schema_version = 1

[ingest_limits]
max_deferred_total = 5
```

остальные лимиты останутся кодовыми.

API:

- `SecurityConfig::load(state_home) -> Result<SecurityConfig, ConfigError>` —
  читает `state_home/config.toml`; при отсутствии файла возвращает кодовые
  defaults (обратная совместимость со старыми per-user каталогами и tempdir-
  тестами);
- `SecurityConfig::default_toml() -> String` — каноническое TOML-представление
  defaults, используется при первичной записи;
- `SecurityConfig::ensure_default_file(state_home) -> Result<(), ConfigError>`
  — записывает defaults, только если файла ещё нет (идемпотентно; не
  перезаписывает пользовательские правки).

Подробное руководство по каждому параметру (смысл, когда менять, последствия)
вынесено в отдельный публичный документ `apps/lltt/SECURITY_CONFIG.md`
(планируется).

### Секция `[log]`: `LogConfig`

`LogConfig` живёт в `liveletters-log` (а не в `liveletters-config`, чтобы разорвать цикл зависимостей между этими двумя крейтами); `liveletters-config` ре-экспортирует тип под именем `liveletters_config::LogConfig`. Полная документация — в `modules/liveletters-log/INTERFACE.md`.

TOML-пример:

```toml
[log]
destination = "file"        # file | stderr | none
level = "info"              # off | error | warn | info | debug | trace
max_size_bytes = 5242880    # 5 МиБ, минимум 1024
keep_files = 3              # количество архивов
include_bodies = false      # true => писать payload через `liveletters_log::log_payload`
```

Из CLI секция управляется командой `lltt settings set log.level info` (и аналогично для остальных полей). По умолчанию журнал выключен (`level = "off"`) и не пишет никуда.

## Имя текущего пользователя liveletters: `read_current_identity` / `write_current_identity`

```rust
pub fn read_current_identity(home: &Path) -> Result<String, ConfigError>
pub fn write_current_identity(home: &Path, name: &str) -> Result<(), ConfigError>
pub fn current_user_path(home: &Path) -> PathBuf
```

Файл `<home>/current-user` — единственный источник истины о том, кто сейчас выбран текущим пользователем liveletters. **Переменная окружения `LLTT_CU` не поддерживается**: ранее она была зарезервирована как fallback, но фактически делала систему неконсистентной (можно было переключиться через `lltt cu`, но `LLTT_CU=другое_имя` продолжал указывать на старое). Всё проходит через файл.

- `read_current_identity(home)`:
  - если `<home>/current-user` существует — читает его, обрезает пробелы, возвращает;
  - если файл отсутствует — возвращает `Err(ConfigError::NoCurrentUser(path))`; это нормальное состояние сразу после `lltt init`, пока пользователь не выполнит `lltt user add` и `lltt cu <имя>`.
- `write_current_identity(home, name)`:
  - записывает `name` в `<home>/current-user` одной строкой;
  - если каталог `home` не существует — возвращает `Err(ConfigError::Io(_))` (`fs::write` не создаёт родительские каталоги).
- `current_user_path(home)` — возвращает `home.join("current-user")`, удобно для диагностических сообщений и тестов.

Запись этого файла — ответственность команды `lltt cu <имя>` (переключение). Чтение — ответственность бинаря `apps/lltt` при построении `CommandContext` и команды `lltt cu`.

## Мост к `AppSettings`: `map_identity_to_settings` / `settings_to_identity`

Эти две функции образуют мост между «дисковой» формой `IdentityConfig` и «оперативной» формой `AppSettings` (из `liveletters-app-core`).

### `map_identity_to_settings(identity: &IdentityConfig) -> AppSettings`

Преобразует `IdentityConfig` в `AppSettings`:

1. начинает с `AppSettings::empty()` (все строки пустые, `setup_completed = false`);
2. копирует `display_name` → `nickname`;
3. копирует `mail.publish` → `email_address`;
4. выставляет `setup_completed = true` (раз конфиг идентичности валиден, first-run уже позади);
5. если задан `SmtpSettings`, копирует host/port/security/username/password в `smtp_*` поля;
6. если задан `ImapSettings`, копирует host/port/security/username/password/mailbox в `imap_*` поля;
7. **не копирует** `resources_owned` и `subscriptions` — у `AppSettings` нет для них полей. Это сознательное усечение: `AppSettings` используется для UI/транспорта, а список ресурсов и подписок хранится исключительно в `IdentityConfig` и читается напрямую.

### `settings_to_identity(settings: &AppSettings) -> IdentityConfig`

Обратное преобразование:

1. `settings.nickname` → `display_name`;
3. `settings.email_address` → `mail.publish`;
4. `receive` заполняется пустым `Vec` (в `AppSettings` нет списка receive-адресов);
5. `smtp` создаётся только если `settings.smtp_host` не пуст; иначе `None`; `pwd_obfuscate` ставится в `true`;
6. `imap` создаётся только если `settings.imap_host` не пуст; иначе `None`; `pwd_obfuscate` ставится в `true`;
7. `meta` инициализируется `Default::default()` — round-trip `map → settings → map2` не сохраняет `resources_owned` и `subscriptions`. Это документированное ограничение: чтобы не потерять данные, не редактируйте `resources_owned`/`subscriptions` через `AppSettings`, а работайте с `IdentityConfig` напрямую.

### `parse_mail_security(&str) -> MailSecurity`

Внутренний хелпер в `mapping.rs`. Маппит `"none"` → `MailSecurity::None`, `"tls"`/`"ssl"`/`"ssl/tls"` → `MailSecurity::Tls`, всё остальное (включая `"starttls"` и пустую строку) → `MailSecurity::StartTls`.

Это намеренно разрешает forward-compat: если в `AppSettings` окажется строковое значение, которого мы ещё не знаем (например, устаревший формат), оно будет интерпретировано как `StartTls` — самый совместимый режим. Парсер не «роняет» конфигурацию из-за незнакомой строки.

## Ошибки: `ConfigError`

```rust
pub enum ConfigError {
    Io(String),
    Toml(String),
    MissingField { field: &'static str },
    UnknownIdentity(String),
    NoCurrentUser(std::path::PathBuf),
}
```

- `Io(String)` — ошибка файловой системы (чтение, запись, создание каталога). Содержит `to_string()` от `std::io::Error`.
- `Toml(String)` — ошибка `toml::from_str` / `toml::to_string_pretty`. Содержит сообщение парсера/сериализатора.
- `MissingField { field: &'static str }` — сейчас не возвращается из публичного API напрямую, но зарезервировано для случая, когда крейт начнёт делать дополнительную валидацию помимо `serde`.
- `UnknownIdentity(String)` — `load_identity` не нашёл файл с указанным именем; в поле лежит само имя, чтобы UI мог показать «не знаю идентичность `alice`».
- `NoCurrentUser(PathBuf)` — `read_current_identity` не нашёл `<home>/current-user`. Сообщение ошибки подсказывает создать и выбрать пользователя через `lltt user init`, `lltt user add` и `lltt cu <имя>`.

Реализован `From<std::io::Error>`, `From<toml::de::Error>`, `From<toml::ser::Error>`, поэтому `?` в вызывающем коде пробрасывает ошибки без обёртки.

`ConfigError` реализует `std::error::Error` вручную (без `#[derive(thiserror::Error)]`); это сознательное упрощение, чтобы крейт не зависел от `thiserror` в публичной поверхности.

## Примеры использования

### Прочитать текущего пользователя liveletters

```rust
use liveletters_config::{load_global, load_identity, read_current_identity};

let home = liveletters_store::StorePaths::from_environment()?.into_home_path();
let _global = load_global(&home)?;

let name = read_current_identity(&home)?;
let identity = load_identity(&home, &name)?;
println!("Текущий пользователь liveletters: {}", identity.display_name());
```

### Сохранить новую идентичность

```rust
use liveletters_config::{save_identity, IdentityConfig, MailSettings, SmtpSettings, MailSecurity};

let identity = IdentityConfig {
    display_name: "Alice".into(),
    mail: MailSettings {
        publish: "alice-publish@example.org".into(),
        receive: vec!["alice-feed@example.org".into()],
        smtp: Some(SmtpSettings {
            host: "smtp.example.org".into(),
            port: 587,
            security: MailSecurity::StartTls,
            username: "alice".into(),
            password: "secret".into(),
            pwd_obfuscate: true,
            hello_domain: "example.org".into(),
        }),
        imap: None,
    },
    meta: Default::default(),
};

save_identity(&home, "alice", &identity)?;
```

### Конвертировать в `AppSettings` и обратно

```rust
use liveletters_config::{map_identity_to_settings, settings_to_identity};

let settings = map_identity_to_settings(&identity);
// … передать settings в transport или UI …

// позже, при сохранении правок:
let updated_identity = settings_to_identity(&settings);
save_identity(&home, "alice", &updated_identity)?;
```

## Что модуль не делает

- не валидирует email-адреса (это транспортный слой);
- не подтверждает пароли через терминал и не открывает хранилище секретов — это делает `lltt user add`; модуль только хранит `password` и `pwd_obfuscate`;
- не принимает имя текущего пользователя из CLI-флагов или переменных окружения (`LLTT_CU` явно не поддерживается, `--as` отложен на будущее);
- не делает миграций `schema_version` (это будущая фича);
- не открывает сетевых соединений.

## Граница с `liveletters-app-core`

`liveletters-config` зависит от `liveletters-app-core` ради одного типа: `AppSettings`. Это сознательная зависимость: «дисковая» и «оперативная» формы — два разных взгляда на одну и ту же конфигурацию, и без общего типа `AppSettings` мост `map_identity_to_settings` / `settings_to_identity` не имел бы точки стыковки.

Обратной зависимости нет: `liveletters-app-core` ничего не знает про TOML-парсинг и не зависит от `liveletters-config`.
