# LiveLetters: технический план работ

Документ описывает поэтапный план реализации минимально рабочей CLI-утилиты `lltt` согласно концепции из [`idea-technical.md`](idea-technical.md). Каждый этап заканчивается проверяемым критерием готовности — конкретной командой `lltt`, которая должна работать.

Все пути к файлам в примерах — относительные, от корня проекта. Где это осмысленно, приводятся короткие фрагменты кода, иллюстрирующие решение.

## Общая методология

В плане соблюдаются три ключевых правила разработки:

1. **Сначала красный тест, потом код, потом зелёный.** Этапы плана формулируются так, чтобы на каждом был свой юнит-тест или интеграционный сценарий.
2. **Документация правится только после зелёных тестов.** Этот план и [`idea-technical.md`](idea-technical.md) — исходные документы; их уточнения делаются по результатам этапов, а не до.
3. **Исходные файлы не превышают 600 строк.** При превышении — разбиваются на тематические файлы. Тесты выносятся в `tests/`.

Тестовая стратегия:

- **БД.** Все тесты используют `tempfile::TempDir` и открывают `Store` через `Store::open_for_home_dir(temp_home)`. Никаких `open_in_memory` (конструктор удаляется).
- **MIME.** Тесты разбора конструируют `ReceivedEmail` напрямую и пишут письма в файлы на диске во временных каталогах.
- **Сеть.** Транспортные тесты с IMAP/SMTP используют локальный почтовый сервер (mailpit) в Docker. Сетевые тесты добавятся в следующей части плана, вместе с командой `lltt sync`.

## Этап 0. Каркас рабочего пространства

**Цель.** Получить Cargo-рабочее пространство, в котором все крейты из `modules/` собраны и тестируются вместе. Бинарного приложения на этом этапе нет.

**Мотивация.** Прежде чем строить `lltt`, нужно убедиться, что фундамент (доменная модель, протокол, БД, разборщик, прикладной сервис, диагностика) вообще собирается в одном дереве зависимостей. Без явного корневого `Cargo.toml` невозможно ни развивать структуру, ни проверять границы между крейтами.

**Задачи.**

1. Создать корневой `Cargo.toml` с `members = ["modules/*", "apps/*"]` (пустые директории пока не включаются, шаблон `["modules/*", "apps/*"]` исключает каталоги без `Cargo.toml`).
2. Создать каталоги `modules/`, `apps/`, `docs/`, и локальные каталоги для заметок разработчика.
3. Создать крейты (исходный код пишется с нуля, на основе зафиксированных в `idea-technical.md` решений по структуре):
   - `liveletters-domain`
   - `liveletters-protocol`
   - `liveletters-store`
   - `liveletters-mail`
   - `liveletters-sync`
   - `liveletters-app-core`
   - `liveletters-diagnostics`
4. Создать заглушки для новых крейтов (содержат только `lib.rs` с `//!`):
   - `modules/liveletters-secret-box`
   - `modules/liveletters-mime`
   - `modules/liveletters-config`
5. Создать `.gitignore` с локальными каталогами для заметок разработчика, `*.local`, `*.local.*`, `target/`.
6. Добавить `LICENSE` (MIT).

**Тесты.**

- `cargo build --workspace` завершается без ошибок.
- `cargo test --workspace` завершается успешно.
- Никакие новые тесты на этом этапе не добавляются.

**Критерий готовности.** Команда `cargo build --workspace` зелёная, `cargo test --workspace` зелёный.

**Файлы-результаты.**

- `Cargo.toml` в корне
- `modules/liveletters-*/Cargo.toml` (10 штук)
- `apps/` (пустой, без `Cargo.toml`)
- `.gitignore`

## Этап 1. Разделение `liveletters-store` и `liveletters-secret-box`

**Цель.** Извлечь обфускацию секретов из `liveletters-store` в отдельный крейт `liveletters-secret-box`. Удалить конструктор `Store::open_in_memory` и перевести все тесты на временный каталог.

**Мотивация.** Обфускация на XChaCha-Poly1305 ортогональна хранению: одни и те же примитивы нужны и для паролей почты, и для будущих OAuth-токенов, и для шифрования локального кеша. Хранить их в крейте про БД — смешение уровней. Кроме того, `liveletters-store` сейчас тащит `chacha20poly1305` и `base64` транзитивно во всё, что просто хочет открыть БД. После разделения `liveletters-store` зависит только от `rusqlite`.

**Задачи.**

1. Создать `modules/liveletters-secret-box/` с зависимостью `chacha20poly1305 = "0.10"` и `base64 = "0.22"`.
2. Перенести `secret_protection.rs` из `liveletters-store` в `liveletters-secret-box/src/codec.rs`.
3. Добавить в `liveletters-secret-box` публичный API:
   - `SecretBox::open(key_path: &Path) -> Result<SecretBox, SecretBoxError>`
   - `SecretBox::obfuscate(plaintext: &str) -> Result<String, SecretBoxError>`
   - `SecretBox::deobfuscate(obfuscated: &str) -> Result<String, SecretBoxError>`
4. В `liveletters-store` заменить прямые вызовы `PasswordObfuscator` на `SecretBox`. Зависимость от `chacha` и `base64` в `liveletters-store/Cargo.toml` удаляется.
5. Удалить `Store::open_in_memory`. Все существующие тесты в `liveletters-store`, использующие его, переводятся на `Store::open_for_home_dir(temp_home)`.
6. Разбить `liveletters-store/src/store.rs` (742 строки) на несколько файлов по тематике: `schema.rs`, `posts.rs`, `comments.rs`, `outbox.rs`, `raw.rs` — каждый не превышает 600 строк.

**Пример теста (юнит на `SecretBox`).**

```rust
#[test]
fn obfuscation_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let key_path = tmp.path().join("k.bin");
    let box_ = SecretBox::open(&key_path).unwrap();
    let token = box_.obfuscate("hunter2").unwrap();
    assert_ne!(token, "hunter2");
    assert_eq!(box_.deobfuscate(&token).unwrap(), "hunter2");
}
```

**Пример теста (юнит на `Store` через временный каталог).**

```rust
#[test]
fn store_initializes_schema() {
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    let _ = store.initialize_schema().unwrap();
    assert!(tmp.path().join("lltt.db").exists());
}
```

**Тесты.**

- `cargo test -p liveletters-secret-box` зелёный.
- `cargo test -p liveletters-store` зелёный; все юниты используют `tempfile::TempDir`.
- `cargo build --workspace` зелёный; `chacha` и `base64` отсутствуют в дереве зависимостей `liveletters-store`.

**Критерий готовности.** `cargo tree -p liveletters-store` показывает только `rusqlite` в прямых зависимостях. Все тесты в `liveletters-store` зелёные и используют временный каталог.

## Этап 2. Разделение `liveletters-mail` и `liveletters-mime`

**Цель.** Извлечь чистый разбор и сборку MIME из `liveletters-mail` в отдельный крейт `liveletters-mime`. Удалить «in-memory» варианты транспорта.

**Мотивация.** `native-tls` — единственная по-настоящему хрупкая зависимость в проекте, и она сидит в одном крейте с кодом, который её не использует. Пока `lltt` работает с импортом файлов (этапы 4–7), реальный IMAP/SMTP не нужен. Если оставить всё в одном крейте, то даже команда `lltt feed` будет линковаться с OpenSSL. После разделения `liveletters-mime` не имеет сетевых зависимостей, и `lltt` собирается без `native-tls`, пока не подключён `liveletters-mail` с признаком `network`.

**Задачи.**

1. Создать `modules/liveletters-mime/` с зависимостью `liveletters-protocol` (по `path = "../liveletters-protocol"`). `mime.rs`, `parser.rs`, `build_protocol_email`, `decode_protocol_message` переезжают в `liveletters-mime`.
2. Типы `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts` переезжают в `liveletters-mime` и переэкспортируются из `liveletters-mail` для обратной совместимости.
3. Удалить `InMemoryImapMailbox` и `InMemorySmtpTransport` из `liveletters-mail`. В крейте остаются только `ConfiguredImapMailbox` и `ConfiguredSmtpTransport`.
4. Добавить в `liveletters-mail/Cargo.toml` признак (feature) `network`, по умолчанию выключенный. Под `network` — реальные транспорты и `native-tls`. Без `network` — крейт содержит только типы (`MailAuth`, `MailSecurity`, `SmtpTransportConfig`, `ImapMailboxConfig`, `SendStatus`, `FetchStatus`, `MailboxCursor`, `FetchBatch`, `MailRetryPolicy`).
5. Разбить `liveletters-mail/src/imap.rs` (283 строки) и `smtp.rs` (248 строк) на файлы: `transport/imap.rs`, `transport/smtp.rs`, `transport/mod.rs`.
6. Тесты разбора (`parse_email`, `extract_liveletters_parts`, `build_protocol_email`) переносятся в `liveletters-mime` и работают с файлами на диске во временных каталогах.

**Пример теста на `liveletters-mime`.**

```rust
#[test]
fn extracts_human_and_technical_parts() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("mail.eml");
    std::fs::write(&path, sample_multipart_mime()).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let parsed = parse_email(&raw).unwrap();
    let parts = extract_liveletters_parts(&parsed).unwrap();
    assert!(!parts.human_readable_body.is_empty());
    let msg: ProtocolMessage = decode_protocol_message(&parts.technical_body).unwrap();
    assert!(!msg.envelope.event_id.is_empty());
}
```

**Тесты.**

- `cargo test -p liveletters-mime` зелёный.
- `cargo build -p liveletters-mail` (без признака `network`) зелёный и **не** линкуется с `native-tls`.
- `cargo build -p liveletters-mail --features network` зелёный.
- В `liveletters-mail` отсутствуют `InMemory*`-типы.

**Критерий готовности.** `cargo tree -p liveletters-mail --no-default-features` не содержит `native-tls`. Все юниты `liveletters-mime` и `liveletters-mail` (без сети) зелёные.

## Этап 3. Крейт `liveletters-config`

**Цель.** Реализовать разбор и сериализацию TOML-конфигов в `~/.liveletters/` в отдельном крейте `liveletters-config`.

**Мотивация.** Конфигурация на диске и рабочие структуры в памяти — разные представления с разным жизненным циклом миграций. Хранить их в `liveletters-app-core` означает привязать доменную модель к текстовому формату и к путям в `$HOME`. Отдельный крейт позволяет позже подключить его из UI без зависимости от всего `app-core`.

**Задачи.**

1. Создать `modules/liveletters-config/` с зависимостями `serde = { version = "1", features = ["derive"] }`, `toml = "0.8"`, `liveletters-app-core` (по `path`).
2. Определить структуры:
   - `GlobalConfig` (содержимое `~/.liveletters/config.toml`);
   - `IdentityConfig` (содержимое `~/.liveletters/identities/<name>.toml`);
   - `MailSettingsFile` (вложенный блок `mail` идентичности);
   - `ResourceSubscription` (одна подписка).
3. Реализовать `load_global(home: &Path) -> Result<GlobalConfig, ConfigError>` и `load_identity(home: &Path, name: &str) -> Result<IdentityConfig, ConfigError>`.
4. Реализовать `save_identity(home: &Path, name: &str, cfg: &IdentityConfig) -> Result<(), ConfigError>`.
5. Реализовать `read_current_identity(home) -> Result<String, ConfigError>` и `write_current_identity(home, name) -> Result<(), ConfigError>` — чтение и запись имени текущего пользователя liveletters в файл `<home>/current-user`. Имя НЕ берётся из CLI-флагов или переменных окружения; единственный источник — файл.
6. Реализовать `map_identity_to_settings(identity: &IdentityConfig) -> AppSettings` и обратное преобразование.
7. Разбить на файлы: `global.rs`, `identity.rs`, `mapping.rs`, `error.rs`, `lib.rs`.

**Пример содержимого `identities/alice.toml` (только иллюстрация, финальная схема — отдельный ADR).**

```toml
account_id = "acct_alice_3kf"
display_name = "Alice"

[mail.publish]
address = "alice-publish@example.org"

[[mail.receive]]
address = "alice-feed@example.org"

[mail.imap]
host = "imap.example.org"
security = "tls"

[mail.smtp]
host = "smtp.example.org"
security = "starttls"

[resources.owned]
ids = ["res_alice_blog"]

[[subscriptions]]
resource_id = "res_bob_blog"
delivery_address = "bob-feed@example.org"
```

**Тесты.**

- `cargo test -p liveletters-config` зелёный.
- Тесты на разбор и сериализацию, на отказ при отсутствии обязательных полей, на маппинг в `AppSettings`.
- Тесты работают через `tempfile::TempDir`.

**Критерий готовности.** Команда `cargo test -p liveletters-config` зелёная; `load_identity` корректно разбирает пример выше.

## Этап 4. CLI-каркас и команды `init`, `cu`, `home`

**Цель.** Появиться бинарю `apps/lltt` с подкомандами `init`, `cu` и `home`. Эти три команды — минимальный проверяемый сценарий, который доказывает, что конфиг, БД и крейт `liveletters-config` правильно соединяются. После этого этапа можно запускать `lltt feed` (этап 6).

**Соглашения CLI-поверхности.** Глобальных флагов у `lltt` нет. Домашний каталог определяется переменной окружения `LIVELETTERS_HOME` (если не задана — `~/.liveletters/`). Имя текущего пользователя liveletters хранится в файле `<home>/current-user` и меняется командой `lltt cu <имя>`. **Переменная `LLTT_CU` и флаг `--as` не поддерживаются** (отложены на будущее). Формат вывода всегда человекочитаемый текст; машиночитаемое представление, если потребуется, добавляется отдельной задачей.

**Задачи.**

1. Создать `apps/lltt/Cargo.toml` с зависимостями: `clap = { version = "4", features = ["derive"] }`, `liveletters-config`, `liveletters-store`, `liveletters-app-core`.
2. Реализовать `src/paths.rs`: одна функция `resolve_home() -> PathBuf` читает `LIVELETTERS_HOME`; если переменная не установлена, возвращает `~/.liveletters/`. Все остальные модули получают путь через неё.
3. Реализовать `src/main.rs` с clap-деревом:
   - подкоманды верхнего уровня: `init`, `cu`, остальные — заглушки;
   - никаких глобальных флагов.
4. Реализовать `src/cli/init.rs`:
   - создаёт каталоги `identities/`, `inbox/`, `outbox-staged/`, `logs/`;
   - создаёт `lltt.db` через `Store::open_for_home_dir` и `initialize_schema`;
   - создаёт `mail-password-obfuscation.key` через `SecretBox::open`;
   - создаёт минимальный `config.toml` и `identities/default.toml` через `liveletters-config`;
   - создаёт файл `current-user` со значением `default`.
5. Реализовать `src/cli/cu.rs` с диспетчером позиционного аргумента и подкоманд:
   - `lltt cu <имя>` — записывает `<имя>` в файл `<home>/current-user`, печатает подтверждение;
   - `lltt cu` без аргумента — печатает имя текущего пользователя liveletters (если файл `<home>/current-user` отсутствует, команда возвращает ошибку `NoCurrentUser`);
   - `lltt cu list` — печатает имена конфигов в `identities/`;
   - `lltt cu show <имя>` — печатает содержимое конфига (с маскированием пароля, если есть);
   - `lltt cu add <имя>` — читает конфиг из файла, указанного через `--from <path>`, и кладёт в `identities/<имя>.toml`;
   - `lltt cu rm <имя>` — удаляет файл `identities/<имя>.toml` (с подтверждением или флагом `--yes`; запрещено удалять того, кто сейчас выбран текущим).
6. Семантика диспетчера для `lltt cu`:
   - если первый аргумент совпадает с именем подкоманды (`list`, `show`, `add`, `rm`) — вызывается соответствующая подкоманда;
   - если первый аргумент — что-то иное, трактуется как имя пользователя, и команда переключает текущего;
   - `lltt cu` без аргументов показывает текущего.
7. Реализовать маскирование секретов: пароль показывается как `********`, пока не запрошен `--reveal`.

**Пример запуска.**

```sh
$ export LIVELETTERS_HOME=~/liveletters-test
$ lltt init
инициализирован /home/user/liveletters-test
создан lltt.db
создан mail-password-obfuscation.key
создан identities/default.toml
создан current-user

$ lltt cu list
default

$ lltt cu add alice --from ./alice.toml
добавлен identities/alice.toml

$ lltt cu alice
текущий пользователь: alice

$ lltt cu show alice
account_id        = "acct_alice_3kf"
display_name      = "Alice"
mail.publish      = "alice-publish@example.org"
mail.receive[0]   = "alice-feed@example.org"

$ lltt cu
alice
```

**Тесты.**

- Юнит-тест: `init` создаёт ожидаемую структуру каталогов, файл `lltt.db` и файл `current-user` со значением `default`.
- Юнит-тест: `lltt cu` показывает `default` после `init`; `lltt cu alice` записывает `alice` в `current-user`; `lltt cu` после переключения показывает `alice`.
- Юнит-тест: `lltt cu` без `init` (на пустом `LIVELETTERS_HOME`) возвращает ошибку `NoCurrentUser` с понятным сообщением и кодом 2.
- Юнит-тест: `lltt cu add` + `lltt cu list` + `lltt cu show` + `lltt cu rm` работают при установленной переменной `LIVELETTERS_HOME` на временный каталог.
- Юнит-тест: при `init` поверх существующего каталога выдаётся ошибка (или требуется флаг `--force`).

**Критерий готовности.** Команды `lltt init`, `lltt cu`, `lltt cu add|list|show|rm` работают при установленной переменной `LIVELETTERS_HOME`. Все тесты зелёные.

**Файлы-результаты.**

Этап 4 реализуется в модульной раскладке: каждая подкоманда `lltt` — отдельный библиотечный крейт в `modules/`, а бинарь `apps/lltt` — тонкий диспетчер над ними. Это отступление от исходного плана (где предполагалось, что код команд лежит в `apps/lltt/src/cli/<command>.rs`); причины — ускорение инкрементальной сборки и изоляция зависимостей между подкомандами.

Бинарь и общие модули:

- `apps/lltt/Cargo.toml` — манифест бинаря, зависит от 13 командных крейтов и `liveletters-config`/`liveletters-store`/`liveletters-output`
- `apps/lltt/src/main.rs` — clap-дерево, диспетчер по `enum Command`, обработка ошибок и exit-кодов
- `apps/lltt/src/context.rs` — `resolve_home()`, `resolve_current_user_name()`, `build_context()` (читает имя ТОЛЬКО из `<home>/current-user`; `LLTT_CU` не используется)
- `apps/lltt/INTERFACE.md` — публичная поверхность бинаря (12 подкоманд, окружение, exit-коды)
- `apps/lltt/TECHNICAL_SPEC.md` — архитектура бинаря

Интеграционные тесты бинаря:

- `apps/lltt/tests/cli_smoke.rs` — `lltt --help` показывает 12 подкоманд; неизвестная подкоманда → ошибка; команда без `init` → код 2
- `apps/lltt/tests/cli_init.rs` — `lltt init` через бинарь
- `apps/lltt/tests/cli_cu.rs` — `lltt cu` (5 операций) через бинарь
- `apps/lltt/tests/cli_default.rs` — default-ветка `<HOME>/.liveletters/` без `LIVELETTERS_HOME`

Реализация содержательных команд (этап 4 по публичному плану):

- `modules/liveletters-init/` — команда `lltt init` (отдельный крейт)
- `modules/liveletters-cu/` — команда `lltt cu` (5 операций: switch, current, list, show, add, rm)
- `modules/liveletters-output/` — общий вывод: `mask_password`, `print_kv`, `print_table`, `print_identity`

Заглушки команд этапов 6+ (содержательная реализация в следующих этапах, не в этапе 4):

- `modules/liveletters-feed/`, `modules/liveletters-inbox/`, `modules/liveletters-post/`, `modules/liveletters-comment/`, `modules/liveletters-outbox/`, `modules/liveletters-thread/`, `modules/liveletters-status/`, `modules/liveletters-doctor/`, `modules/liveletters-settings/`, `modules/liveletters-lltt-sync/`

## Этап 5. Подписки и фильтрация входящих

**Цель.** Реализовать команду `lltt sub` (управление подписками) и фильтрацию входящих `PostCreated`/`CommentCreated` на этапе `sync` по списку подписок идентичности. Доставить короткий подписной протоколь `subscription_changed` через `outbox` → `sync push`.

**Мотивация.** Без подписок на этапе синхронизации пользователь получает в ленту чужие посты всех блогов подряд. Подписки нужны до этапа 6 (`feed`), чтобы лента к моменту показа уже была отфильтрована.

**Задачи.**

1. Ввести value-object `liveletters_domain::ResourceAddress` (адрес блога = `mail.publish` другого пользователя; валидация: наличие `@`, непустая локальная часть, недопустимы пробелы).
2. Ввести `liveletters_domain::Subscription { resource_address, subscriber_account_id, subscriber_delivery_address }` и `SubscriptionAction { Subscribe, Unsubscribe }` с `as_str()`.
3. Ввести событие `SubscriptionChanged` (envelope `event_type = "subscription_changed"`, payload — 5 полей, описанных в плане). Добавить вариант `DomainEventPayload::SubscriptionChanged` в `liveletters-protocol`.
4. В `liveletters-config` поменять тип `meta.subscriptions` с `Vec<ResourceSubscription>` на `Vec<ResourceAddress>`; в TOML это выражается как `subscriptions = ["…", "…"]` под `[meta]`. Удалить `ResourceSubscription`.
5. В `liveletters-store` создать таблицу `subscriptions(resource_address, subscriber_account_id, subscriber_delivery_address, PK(resource_address, subscriber_account_id))` + 4 метода (`save_subscription`, `delete_subscription → Result<bool>`, `list_subscriptions_for_resource`, `list_subscriptions_for_subscriber`) + `SubscriptionRecord`.
6. В `liveletters-app-core` ввести команды `subscribe`/`unsubscribe` (каждая атомарно пишет в `subscriptions` + enqueue 1 outbox) и query `ListSubscriptionsQuery` (без зависимости от `liveletters-config`).
7. В `liveletters-sync` добавить `SyncEngine::new_with_identity(store, own_address, &[subscribed])`, обработать `SubscriptionChanged` в `apply_payload` и отфильтровать `PostCreated`/`CommentCreated` если `resource_id` не равен `own_address` и не входит в `subscribed`. Фильтр записывает `raw_events` с `apply_status="filtered"` и `failure_reason="not_subscribed"` и возвращает `SyncMessageOutcome::Filtered { message_id, event_id, reason }`. В `SyncReport`/`DeferredReprocessingSummary` добавить поле `filtered`.
8. Создать крейт `liveletters-sub` (`Cargo.toml`, `src/lib.rs`, `src/args.rs`, `src/run.rs`, `src/error.rs`, `INTERFACE.md`, `TECHNICAL_SPEC.md`) с операциями `subscribe`/`list`/`rm`. `delivery_address` берётся из `mail.receive[0]`, фолбэк — `mail.publish`. Короткое имя — `sub`.
9. В `apps/lltt` добавить `Command::Sub(liveletters_sub::Args)` + 3 интеграционных теста в `apps/lltt/tests/cli_sub.rs`. Обновить `INTERFACE.md` бинаря.
10. Переименовать последующие этапы плана (5→6, 6→7, 7→8, 8→9, 9→10) и обновить сводку.

**Критерий готовности.** `cargo test --workspace` зелёный (≈200 тестов), `cargo clippy --workspace` без новых предупреждений. `lltt sub <адрес>` добавляет подписку в `meta.subscriptions` и в таблицу `subscriptions`, кладёт событие в `outbox`. `lltt sub list` печатает таблицу. `lltt sub rm <адрес>` удаляет и в TOML, и в БД, и пишет событие с `action=unsubscribe`. В `sync` `PostCreated` от чужого блога, на который нет подписки, попадает в `raw_events` с `apply_status="filtered"` и НЕ появляется в ленте.

## Этап 6. Команда `lltt feed` — минимально рабочий продукт

**Цель.** Реализовать импорт одного протокольного сообщения из файла и вызов `lltt feed` для его отображения.

**Мотивация.** Это центральный сценарий: «электронное письмо → БД → лента». Без него нельзя сказать, что система вообще работает. Этап специально минимален: одна команда, один путь файла, один пост в ленте. Все остальные сценарии строятся вокруг этого стержня.

**Задачи.**

1. Реализовать `src/cli/inbox.rs` с подкомандой `import`:
   - `lltt inbox import <file.eml>` — читает файл, прогоняет через `SyncEngine::ingest_batch`, печатает сводный отчёт `SyncReport` (`Applied | Duplicate | ...`).
2. Реализовать `src/cli/feed.rs`:
   - открывает `Store` через `open_for_home_dir`;
   - создаёт `AppCore::new(&store)`;
   - вызывает чтение постов, подходящих под подписки текущей идентичности;
   - печатает ленту подписок в человекочитаемом виде.
3. В `liveletters-app-core` подтвердить, что запросы чтения постов принимают идентичность как параметр (а не выводят её из контекста).
4. Покрыть путь: написать в тестах временный файл `.eml` с валидным протокольным сообщением, импортировать его, вызвать `lltt feed`, проверить, что в ленте появился пост.

**Пример запуска.**

```sh
$ lltt inbox import sample.eml
применено: 1
отложено:   0
отклонено:  0

$ lltt feed
┌─ пост #post_abc от acct_alice_3kf
│  Привет, это первый пост.
│  2026-06-04 14:20
└─
```

**Тесты.**

- Интеграционный тест в `apps/lltt/tests/cli_feed.rs`:
  - инициализация временного `LIVELETTERS_HOME`;
  - создание тестового `.eml` с валидным `ProtocolMessage`;
  - вызов `lltt inbox import` через бинарь (через `assert_cmd` или `espresso`-паттерн);
  - вызов `lltt feed` и проверка, что пост появился.
- Юнит-тест: повторный импорт того же файла даёт `Duplicate`, а не `Applied`.

**Критерий готовности.** После `lltt inbox import sample.eml` команда `lltt feed` печатает непустую ленту. Это и есть минимально рабочий продукт.

## Этап 7. Команды чтения и записи

**Цель.** Реализовать `lltt post new`, `lltt comment new`, `lltt thread`, `lltt outbox list`.

**Мотивация.** После этапа 6 пользователь может только читать. Минимально рабочий продукт подразумевает и публикацию: пост или комментарий должны создаваться через CLI и попадать в outbox (и применяться локально, чтобы лента обновилась немедленно). Это превращает `lltt` из «считывателя» в инструмент, через который можно вести блог.

**Задачи.**

1. Расширить `AppCore::CreatePostCommand` и `CreateCommentCommand` полем `visibility: Visibility`. В CLI поддерживаются только `public` и `friends_only`; остальные уровни (`members_only`, `private_community`) — отложенная задача.
2. Ввести в `liveletters-app-core` модуль `ids` с функциями `new_post_id() -> "post-<unix_millis>"` и `new_comment_id() -> "comment-<unix_millis>"`. Коллизии при ручном вводе исключены (тесты передают готовые ID), а для пользовательского темпа создания записей миллисекундного разрешения достаточно.
3. Ввести в `liveletters-app-core` value-object `Identity { account_id, publish }` (минимальный, без зависимости от `liveletters-config`) и хелперы `create_post_from_identity` / `create_comment_from_identity`. Хелперы подставляют `post_id`/`comment_id` (через генератор), `created_at = unix_millis_now() / 1000`, `resource_id = identity.publish`, `author_id = identity.account_id`.
4. В `liveletters-output` добавить общие утилиты: `read_body(Option<&Path>, &mut Read) -> Result<String, String>` (читает из файла или stdin) и `parse_visibility(&str) -> Result<Visibility, String>` (`public` | `friends_only`).
5. Реализовать команды:
   - `lltt post new [--body-file <path>] [--visibility <level>]` — читает тело, валидирует видимость, делегирует в `AppCore::create_post_from_identity`. Печатает `запись создана: <post_id>`.
   - `lltt comment new --post <id> [--parent <id>] [--body-file <path>] [--visibility <level>]` — то же для комментария. Печатает `комментарий создан: <comment_id>`. Если пост или родитель не найдены, `AppCore` возвращает `PostNotFound` / `CommentNotFound`.
   - `lltt thread <post_id>` — печатает пост и дерево комментариев. Дерево строится по `parent_comment_id`; сортировка внутри уровня — по `created_at`. Префиксы: `•` для корня, `↳` для ответа. Использует `AppCore::get_post_thread`.
   - `lltt outbox list` — печатает `PendingOutbox` через `AppCore::get_pending_outbox` в виде таблицы (`event_id | event_type | resource_id`). Команда read-only: ничего не пишет в БД.

**Пример запуска.**

```sh
$ echo "Привет, мир." | lltt post new
запись создана: post-1717161234567

$ lltt thread post-1717161234567
┌─ пост #post-1717161234567 от default
│  visibility: public
│  Привет, мир.
└─

комментарии: 0

(нет комментариев)

$ lltt outbox list
неотправленные события: 1

event_id                          event_type        resource_id
post-created:post-1717161234567   post_created      default-publish@example.org
```

**Тесты.**

- Юнит-тесты в `liveletters-app-core` (новые файлы `tests/create_post.rs`, `tests/create_comment.rs`, `tests/from_identity.rs`, `tests/from_identity_comment.rs`): видимость `friends_only` сохраняется в БД и попадает в `outbox`; хелперы подставляют поля корректно и генерируют уникальные ID.
- Юнит-тесты `src/ids.rs::tests` (4): генерация ID через `unix_millis`, неубывание соседних значений.
- Юнит-тесты в `liveletters-post` (6 unit) и `liveletters-comment`/`liveletters-outbox`/`liveletters-thread`: разбор аргументов, парсинг видимости, чтение тела из файла и stdin, обработка ошибок.
- Интеграционные тесты в `tests/flow.rs` каждого командного крейта (4 + 5 + 3 + 5 = 17): `run(ctx, args)` end-to-end через `Store` на временном каталоге.
- Покрытие через бинарь в `apps/lltt/tests/cli_post.rs` (2), `cli_comment.rs` (2), `cli_outbox.rs` (2), `cli_thread.rs` (2): `init` + команды + SQL-проверка состояния БД.

**Критерий готовности.** Команды `lltt post new`, `lltt comment new`, `lltt thread`, `lltt outbox list` работают при установленной переменной `LIVELETTERS_HOME` на временный каталог. End-to-end сценарий `init → post new → feed → thread → comment new → outbox list` проходит без ошибок. `cargo test --workspace` зелёный (≥ 250 тестов).

## Этап 8. Диагностика и наблюдаемость

**Цель.** Реализовать команды `lltt status`, `lltt doctor`, `lltt inbox list`, `lltt settings show|set` — «приборную панель» для пользователя и разработчика.

**Мотивация.** Без диагностики нельзя понять, почему событие не применилось или почему outbox не отправляется. Эти команды не добавляют новой логики синхронизации, а только наблюдают за состоянием БД. Деление на `status` и `doctor` — это разные уровни: `status` показывает бизнес-счётчики (что лежит в `posts`/`comments`/`outbox`), `doctor` — счётчики синхронизации (что лежит в `raw_messages` и как распределено по категориям).

**Слои архитектуры.**

```
apps/lltt
   └─ clap-разбор → <command>::run(&ctx, &args)
                     │
        ┌────────────┼────────────┬─────────────────┐
        ▼            ▼            ▼                 ▼
   liveletters-  liveletters-  liveletters-      liveletters-
     status       doctor        inbox              settings
        │            │            │                  │
        │            │            │                  │
        ▼            ▼            ▼                  ▼
            liveletters-store    liveletters-     liveletters-store
            (count_*,            inbox::list     (update_*_settings_
             newest_post_         фильтрует         field,
             created_at,          raw_messages      obfuscate_secret)
             list_raw_msgs)
                                  liveletters-
                                    diagnostics
                                  (DiagnosticsReader)
```

Четыре командных крейта; общая зависимость — `liveletters-store`. Дополнительно: `liveletters-doctor` зависит от `liveletters-diagnostics`, `liveletters-settings` — от `liveletters-config` (для `read_current_identity`).

**Задачи.**

1. `lltt status` — печатает пять полей: `постов`, `комментариев`, `отложенных`, `исходящих`, `последняя активность`. Источник — четыре SQL `COUNT(*)` по таблицам `posts`/`comments`/`deferred_events`/`outbox` плюс `MAX(created_at)` из `posts` (NULL → `нет активности`). Дата форматируется через `liveletters_output::format_unix_iso8601_utc` как `YYYY-MM-DD HH:MM:SS UTC`. Ограничение форматтера: диапазон 1970–2100; для `created_at > 2_504_889_600` (`2100-01-01 00:00:00 UTC`) возвращается `2100-01-01 00:00:00 UTC` (сознательное упрощение без `chrono`).
2. `lltt doctor` — печатает `DiagnosticsSnapshot` через `DiagnosticsReader::build_snapshot`. Девять полей: `здоровье` (Healthy, если `malformed + unauthorized + invalid + deferred == 0`, иначе Degraded), семь категорий входящих (`Applied | Duplicate | Replay | Unauthorized | Invalid | Malformed | Deferred`) и `outbox` (размер исходящей).
3. `lltt inbox list [--status <категория>] [--limit <N>]` — показывает последние N строк таблицы `raw_messages` через `Store::list_raw_message_records`. Фильтр `--status` ограничен шестью значениями (`applied`, `duplicate`, `replay`, `unauthorized`, `invalid`, `malformed`); прочие → `InboxError::InvalidStatus`. `--limit` по умолчанию 20. Таблица: `message_id | status | preview` (первая непустая строка `raw_message`, до 80 символов с `…`).
4. `lltt settings show` (или `lltt settings` без подкоманды) — печатает содержимое таблиц `user_settings` и `mail_settings` через `print_kv`. Пароли маскируются как `********` (без `--reveal`). Если запись отсутствует — печатается `[user_settings] отсутствует`.
5. `lltt settings set <ключ> <значение>` — валидирует ключ по жёсткому списку `ALLOWED_KEYS` (16 значений) и обновляет одно поле в БД. Пароли (`smtp.password`, `imap.password`) проходят через `SecretBox::obfuscate` и сохраняются как `obf:v1:…`. При первом `set` соответствующая запись создаётся с дефолтными значениями через `ensure_records_exist`.

**Пример запуска.**

```sh
$ lltt status
постов:           3
комментариев:     5
отложенных:       0
исходящих:        1
последняя активность: 2026-06-04 14:35

$ lltt doctor
здоровье:           Healthy
Applied:            12
Duplicate:          1
Replay:             0
Unauthorized:       0
Invalid:            0
Malformed:          0
Deferred:           0
Outbox (исходящих): 1

$ lltt inbox list --status applied --limit 5
всего: 12
фильтр: applied
показано: 5

message_id               status    preview
01HXYZ…                  applied   Re: завтра
01HXYW…                  applied   Fwd: заметки
…

$ lltt settings
[user_settings]
profile_id:        alice
nickname:          Alice
email_address:     alice@example.org
avatar_url:        —
setup_completed:   true

[mail_settings]
smtp.host:         smtp.example.org
smtp.port:         587
smtp.security:     starttls
smtp.username:     alice
smtp.password:     ********
smtp.hello_domain: example.org
imap.host:         imap.example.org
imap.port:         993
imap.security:     tls
imap.username:     alice
imap.password:     ********
imap.mailbox:      INBOX

$ lltt settings set smtp.host smtp2.example.org
сохранено: mail_settings.smtp.host

$ lltt settings set smtp.password s3cret
сохранено: mail_settings.smtp.password
```

**Дополнения в `liveletters-store`.**

- `count_posts`, `count_comments`, `count_outbox`, `count_deferred_events` — четыре SQL `COUNT(*)`.
- `newest_post_created_at() -> Result<Option<u64>, StoreError>` — `MAX(created_at)` через `Option<i64>` аннотацию `row.get(0)` (NULL → `None`).
- `update_user_settings_field(profile_id, field, value)` — четыре поля: `nickname`, `email_address`, `avatar_url`, `setup_completed`. Запись создаётся лениво через `ensure_records_exist`.
- `update_mail_settings_field(profile_id, field, value)` — двенадцать полей: 6 `smtp.*` + 6 `imap.*`. Пароли — через отдельный `update_mail_password`, который вызывает `obfuscate_secret_if_needed` из `secret_bridge`.
- `StoreError::InvalidColumn(String)` — отказ при попытке записать неизвестное поле.

**Дополнения в `liveletters-diagnostics`.**

- `SyncHealth` переведён на приватные поля + `SyncHealth::new(...)` конструктор + 9 геттеров (`status`, `applied_messages`, `duplicate_messages`, `replayed_messages`, `unauthorized_messages`, `invalid_messages`, `malformed_messages`, `deferred_events`, `pending_outbox`). Это закрывает `SyncHealth.status` от коллизии с `HealthStatus` и инкапсулирует мутабельность.

**Тесты.**

- Юнит-тесты в `liveletters-store/tests/counts.rs` (6): `count_posts`/`count_comments`/`count_outbox`/`count_deferred_events` после серии импортов + `newest_post_created_at` (`Some(_)` / `None`).
- Юнит-тесты в `liveletters-status/tests/flow.rs` (4): `print_status` на пустой и непустой БД, `StatusError::Store` (read-only БД), `format_unix_iso8601_utc` для 0/1717161234/будущей даты.
- Юнит-тесты в `liveletters-doctor/tests/flow.rs` (4): `print_doctor` на пустой БД (`здоровье: Healthy`), после серии импортов (счётчики совпадают с реальным `raw_messages`), Degraded-ветка (после отклонённого письма), `DoctorError::Store` (read-only БД).
- Юнит-тесты в `liveletters-inbox/tests/list.rs` (4): пустой список, фильтр `applied`, `--limit` меньше числа строк, невалидный статус (`nonsense` → `InvalidStatus`).
- Юнит-тесты в `liveletters-settings/tests/flow.rs` (9): `show` пустой/с записью, `set nickname/email_address/smtp.host/smtp.port/smtp.security/smtp.password` (включая проверку обфускации через прямой SQL-запрос к `mail_settings.smtp_password` — значение начинается с `obf:v1:`), `set bogus.key` → `InvalidKey`, `set` без `init` → ошибка.
- Интеграционные тесты в `apps/lltt/tests/cli_status.rs` (1), `cli_doctor.rs` (1), `cli_inbox_list.rs` (3), `cli_settings.rs` (2) — end-to-end через бинарь после `lltt init`.
- `cli_smoke.rs::status_succeeds_after_init` — ранее `command_when_init_succeeded_returns_not_implemented`; переименован, потому что команда больше не возвращает «не реализовано», а печатает счётчики.

**Критерий готовности.** Все четыре команды работают через бинарь. `cargo test --workspace` зелёный (306 тестов; +34 относительно конца предыдущей части). Счётчики в `doctor` совпадают с фактическим состоянием `raw_messages`. Пароль, записанный через `set smtp.password`, читается через `get_mail_settings_record` как plaintext (после автоматической деобфускации), а в сырой таблице БД — в форме `obf:v1:…`.

## Этап 9. Сетевая синхронизация

**Цель.** Реализовать `lltt sync pull` (IMAP) и `lltt sync push` (SMTP).

**Мотивация.** До этого этапа `lltt` работает с импортом файлов. Это удобно для разработки и отладки, но в реальной жизни пользователь ожидает, что подписки «просто работают»: письма приходят сами, новые посты подписчиков появляются в ленте. На этом этапе включается `liveletters-mail` с признаком `network`, и `lltt` становится end-to-end-инструментом.

**Задачи.**

1. В `apps/lltt/Cargo.toml` добавить `liveletters-mail` с `features = ["network"]`.
2. Реализовать `src/cli/sync.rs`:
   - `lltt sync pull` — открывает IMAP через `ConfiguredImapMailbox`, вытаскивает новые письма, прогоняет через `SyncEngine::ingest_batch`, печатает отчёт;
   - `lltt sync push` — читает outbox, отправляет каждое письмо через `ConfiguredSmtpTransport`, при успехе помечает как отправленное.
3. Добавить `mailpit` в `docker-compose.test.yml` (или в скрипт `scripts/test-mail.sh`): поднимает IMAP и SMTP на `localhost:1110` и `localhost:1025`.
4. Добавить транспортные тесты: импорт через `ConfiguredImapMailbox` в `mailpit` и проверка, что `lltt sync pull` доставил письмо.
5. В `liveletters-mail` тесты, которые раньше использовали `InMemory*`, заменяются на интеграционные с `mailpit`.

**Пример запуска.**

```sh
$ lltt sync pull
подключено к imap.example.org
получено писем:       7
применено событий:     5
дубликатов:            2

$ lltt sync push
подключено к smtp.example.org
отправлено писем:      1
ошибок отправки:       0
```

**Тесты.**

- Интеграционный тест с `mailpit`: поднимается контейнер, отправляется письмо напрямую в его SMTP, вызывается `sync pull`, проверяется, что событие попало в БД.
- Тест идемпотентности: повторный `sync pull` не создаёт дубликатов.

**Критерий готовности.** На тестовом стенде с `mailpit` команда `lltt sync pull` доставляет письма, `lltt sync push` отправляет outbox. Документация по настройке SMTP/IMAP добавлена в `docs/setup-mail.md`.

## Этап 10. Пользовательский интерфейс (отдельное решение)

**Цель.** Принять решение о технологии UI и реализовать его поверх `lltt`.

**Мотивация.** UI не должен появляться, пока `lltt` не закрывает все ключевые сценарии end-to-end. На этом этапе прикладная логика полностью отделена от CLI; UI может быть написан в виде отдельного приложения, вызывающего те же функции `liveletters-app-core`, что и `lltt`.

**Задачи.** Этот этап не входит в первоначальный план минимально рабочего продукта. После успешного прохождения этапов 0–8 создаётся отдельный документ `docs/ui-decision.md`, в котором фиксируется выбор стека (Tauri, обёртка над `lltt`, или веб-интерфейс с локальным сервером). Сам этап реализации UI идёт отдельной задачей.

**Критерий готовности.** Документ `docs/ui-decision.md` с обоснованным выбором стека. Дальнейшие действия — по нему.

## Сводка

| Этап | Что сделано | Критерий |
|---|---|---|
| 0 | Рабочее пространство, 10 крейтов | `cargo build --workspace` |
| 1 | `liveletters-secret-box` отдельно, без in-memory БД | `cargo tree -p liveletters-store` |
| 2 | `liveletters-mime` отдельно, без in-memory транспорта | `cargo tree -p liveletters-mail --no-default-features` |
| 3 | `liveletters-config` | `cargo test -p liveletters-config` |
| 4 | CLI-каркас, `lltt init`, `lltt cu` | `lltt cu list` |
| 5 | `liveletters-sub` + фильтр входящих в `sync` | `lltt sub list`; `PostCreated` от чужого блока фильтруется |
| 6 | `lltt feed` (минимально рабочий продукт) | `lltt feed` после `inbox import` |
| 7 | `lltt post new`, `comment new`, `thread`, `outbox list`; ID-генератор и `Identity`-хелперы в `app-core`; утилиты `read_body` и `parse_visibility` в `liveletters-output` | end-to-end `init → post new → feed → thread → comment new → outbox list` |
| 8 | `lltt status`, `lltt doctor`, `lltt inbox list`, `lltt settings show\|set` | `lltt doctor` показывает корректные счётчики; `lltt settings set smtp.password s3cret` сохраняет `obf:v1:…` |
| 9 | `lltt sync pull`, `lltt sync push` через mailpit | `lltt sync pull` |
| 10 | Решение по UI | `docs/ui-decision.md` |

Каждый этап начинается с красных тестов и заканчивается зелёными. Документация правится только после зелёных тестов соответствующего этапа.
