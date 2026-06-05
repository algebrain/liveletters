# `liveletters-mime` INTERFACE

## Назначение

`liveletters-mime` это отдельный крейт, отвечающий за всё, что связано с MIME-формой LiveLetters-писем: парсинг сырого текста письма, извлечение из multipart-тела человекочитаемой и технической частей, а также сборка исходящего письма из `ProtocolMessage`.

Крейт намеренно отделён от `liveletters-mail` по двум причинам:

- MIME-логика не зависит от того, как именно мы отправляем и получаем письма (SMTP, IMAP, локальный mock, TcpListener) и нужна тестам даже тогда, когда сетевой transport отключён (feature `network` в `liveletters-mail` по умолчанию выключена);
- MIME-логика зависит только от `liveletters-protocol`, но не от `liveletters-store` или `liveletters-app-core`, что держит её переносимой и пригодной для повторного использования в CLI и в Tauri-клиенте.

Крейт занимает одну конкретную нишу: превращать «текст письма в стандарте RFC 5322 / multipart» в строго типизированные структуры LiveLetters и обратно. Он не делает:

- отправку или приём писем (это `liveletters-mail`);
- хранение писем в БД (это `liveletters-store`);
- разбор нестандартных MIME-деревьев, вложений, кодировок base64/quoted-printable в теле (текущая версия работает только с `text/plain` + `application/json` внутри multipart/mixed).

## Где находится интерфейс

- crate: `liveletters-mime`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- `parse_email(&str) -> Result<ParsedEmail, MimeError>`;
- `extract_liveletters_parts(&ParsedEmail) -> Result<ExtractedMailParts, MimeError>`;
- `build_protocol_email(from, to, subject, &ProtocolMessage) -> Result<OutgoingEmail, MimeError>`;
- `decode_protocol_message(&str) -> Result<ProtocolMessage, MimeError>`;
- типы `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts`;
- тип ошибки `MimeError`;
- функция-хелпер `crate_name() -> &'static str` для диагностики.

Внутренние модули `build`, `error`, `message`, `mime`, `parser` не публикуются.

## Что считается внешним интерфейсом этого модуля

С практической точки зрения внешний интерфейс `liveletters-mime` это:

1. четыре функции: `parse_email`, `extract_liveletters_parts`, `build_protocol_email`, `decode_protocol_message`;
2. четыре структуры данных: `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts`;
3. `MimeError` как единый тип ошибок MIME-слоя.

Именно этим API пользуются:

- `liveletters-mail` (реэкспортирует наружу и использует внутри `transport/`);
- integration-тесты `liveletters-mail/tests/mail_flow.rs` и `liveletters-mime/tests/parse.rs`;
- будущий CLI `lltt send` и `lltt ingest` в `apps/lltt`.

## Конвейер: от сырого письма к `ProtocolMessage`

Путь входящего письма через крейт — это явный конвейер из трёх шагов:

```
raw bytes → parse_email → ParsedEmail → extract_liveletters_parts → ExtractedMailParts
                                                                          │
                                                                          └─→ decode_protocol_message(parts.technical_body())
                                                                                          │
                                                                                          └─→ ProtocolMessage
```

Каждый шаг — отдельная функция с собственным `Result`-типом, и каждый шаг можно использовать по отдельности.

### Шаг 1: `parse_email`

```rust
pub fn parse_email(raw_email: &str) -> Result<ParsedEmail, MimeError>
```

Разделяет «сырое письмо» на блок заголовков и тело:

- нормализует CRLF → LF, чтобы не зависеть от того, в какой форме пришло письмо;
- требует, чтобы между последним заголовком и телом была пустая строка (`\n\n`) — это канонический разделитель заголовков и тела в RFC 5322;
- разбирает каждую строку заголовка как `name: value` (с триммингом пробелов вокруг `:`);
- возвращает `MimeError::InvalidEmailFormat` на любом структурном отклонении.

`ParsedEmail` инкапсулирует `headers: Vec<(String, String)>` и `body: String`. Заголовки хранятся в исходном порядке, но поиск по имени — case-insensitive (`eq_ignore_ascii_case`), как и требует RFC 5322.

Полезные методы `ParsedEmail`:

- `body() -> &str` — сырое тело письма;
- `header(name) -> Option<&str>` — первый заголовок с подходящим именем (case-insensitive);
- `subject() -> Option<String>` — то же, что `header("Subject").to_owned()`.

### Шаг 2: `extract_liveletters_parts`

```rust
pub fn extract_liveletters_parts(parsed: &ParsedEmail) -> Result<ExtractedMailParts, MimeError>
```

Извлекает из `ParsedEmail` LiveLetters-специфичную структуру:

- требует заголовок `Content-Type` со значением, содержащим подстроку `multipart/` (любой `multipart/*`, но в нашей форме это `multipart/mixed`);
- парсит параметр `boundary="…"` (поддерживает и границу в кавычках, и без);
- разбивает тело по маркеру `--{boundary}`;
- в каждой части ищет `Content-Type`:
  - если в нём есть `text/plain` — это `human_readable_body`;
  - если в нём есть `application/json` — это `technical_body`;
  - остальные части игнорируются;
- возвращает `MimeError::MissingHumanReadablePart` / `MimeError::MissingTechnicalPart`, если соответствующая часть не найдена.

`ExtractedMailParts` предоставляет:

- `human_readable_body() -> &str`;
- `technical_body() -> &str`.

Тело, которое попадает в `technical_body`, — это JSON-сериализованный `ProtocolMessage` (см. следующий шаг).

### Шаг 3: `decode_protocol_message`

```rust
pub fn decode_protocol_message(input: &str) -> Result<ProtocolMessage, MimeError>
```

Тонкая обёртка над `liveletters_protocol::decode_message`, которая:

- декодирует JSON в `ProtocolMessage`;
- транслирует `ProtocolError` в `MimeError::Protocol(String)` с человекочитаемой формулировкой (`blank envelope field: <name>`, `blank human readable body`, `malformed json: <…>`).

Эта функция нужна, чтобы верхние слои не импортировали `liveletters_protocol` только ради одной строчки маппинга ошибок.

## Сборка исходящего письма: `build_protocol_email`

```rust
pub fn build_protocol_email(
    from: &str,
    to: &str,
    subject: &str,
    protocol_message: &ProtocolMessage,
) -> Result<OutgoingEmail, MimeError>
```

Собирает сырое multipart-письмо из `ProtocolMessage`:

1. сериализует `protocol_message` в JSON через `liveletters_protocol::encode_message`; ошибка протокола → `MimeError::Protocol(…)`;
2. формирует заголовки `From` / `To` / `Subject` / `MIME-Version` / `Content-Type: multipart/mixed; boundary="liveletters-boundary"`;
3. формирует две части:
   - `text/plain; charset="utf-8"` с `protocol_message.human_readable_body()`;
   - `application/json` с сериализованным `ProtocolMessage`;
4. возвращает `OutgoingEmail { from, to, subject, raw_message }`, где `raw_message` — готовое к отправке тело письма целиком (вместе с заголовками и разделителями).

Граница `liveletters-boundary` зафиксирована в коде как константа. Она намеренно детерминирована, чтобы сделать round-trip `build_protocol_email` → `parse_email` → `extract_liveletters_parts` → `decode_protocol_message` воспроизводимым в тестах.

`OutgoingEmail` — это плоская структура с четырьмя `String`-полями:

- `from`, `to`, `subject` — для SMTP envelope;
- `raw_message` — готовое к отправке тело письма.

## Тип `ReceivedEmail`

```rust
pub struct ReceivedEmail {
    pub message_id: String,
    pub raw_message: String,
}
```

Используется как входная точка для IMAP-слоя в `liveletters-mail`. `message_id` приходит из IMAP `UID` или `Message-ID`-заголовка, `raw_message` — это сырой RFC 5322-текст письма, который потом пойдёт в `parse_email`.

`ReceivedEmail` намеренно отделён от `ParsedEmail` и `ExtractedMailParts`: до парсинга мы не знаем, валидно ли письмо и содержит ли оно LiveLetters-структуру. Это разделение позволяет транспортному слою работать с «сырыми» входящими письмами, а MIME-разбор делать ровно там, где он нужен.

## Ошибки: `MimeError`

```rust
pub enum MimeError {
    Protocol(String),
    InvalidEmailFormat(&'static str),
    MissingHumanReadablePart,
    MissingTechnicalPart,
}
```

Смысл вариантов:

- `Protocol(String)` — сбой на стыке с `liveletters_protocol`: пустое поле envelope, пустое `human_readable_body`, битый JSON.
- `InvalidEmailFormat(&'static str)` — структурная проблема MIME: нет `\n\n` между заголовками и телом, заголовок без `:`, отсутствует `Content-Type`, не multipart, нет параметра `boundary`.
- `MissingHumanReadablePart` — в multipart-теле не нашлось `text/plain`.
- `MissingTechnicalPart` — в multipart-теле не нашлось `application/json`.

`MimeError` намеренно **не несёт** подробностей «в каком байте сломались заголовки»: текущая модель ошибок фиксирует класс проблемы, а не позицию. Это сознательное упрощение: LiveLetters не пытается быть почтовым клиентом общего назначения и не предоставляет пользователю средств ручного восстановления битых писем.

`From<MimeError> for liveletters_mail::TransportError` объявлен в `liveletters-mail::lib.rs`, чтобы транспортный слой мог возвращать `MimeError` наверх без `map_err` на каждом шаге.

## Примеры использования

### Собрать письмо и распарсить его обратно

```rust
use liveletters_mime::{build_protocol_email, parse_email, extract_liveletters_parts, decode_protocol_message};
use liveletters_protocol::{MessageEnvelope, ProtocolMessage, DomainEventPayload};

let message = ProtocolMessage::new(
    MessageEnvelope::new("1", "post_created", "blog-1", "event-1")?,
    "Новая запись в блоге",
    DomainEventPayload::PostCreated {
        post_id: "post-1".into(),
        resource_id: "blog-1".into(),
        actor_id: "alice".into(),
        created_at: 1_710_000_000,
        visibility: "public".into(),
    },
)?;

let outgoing = build_protocol_email(
    "alice@example.test",
    "bob@example.test",
    "Новая запись",
    &message,
)?;

let parsed = parse_email(&outgoing.raw_message)?;
let parts = extract_liveletters_parts(&parsed)?;
let decoded = decode_protocol_message(parts.technical_body())?;

assert_eq!(decoded.human_readable_body(), "Новая запись в блоге");
```

### Обработать письмо, пришедшее из IMAP

```rust
use liveletters_mime::{parse_email, extract_liveletters_parts, decode_protocol_message, ReceivedEmail};

let received: ReceivedEmail = /* … из IMAP … */;

let parsed = parse_email(&received.raw_message)?;
match extract_liveletters_parts(&parsed) {
    Ok(parts) => {
        let message = decode_protocol_message(parts.technical_body())?;
        // … передать в liveletters-app-core …
    }
    Err(MimeError::MissingTechnicalPart) => {
        // … это не LiveLetters-письмо, проигнорировать …
    }
    Err(error) => return Err(error.into()),
}
```

## Что модуль не делает

- не разбирает вложения, base64-encoded body, quoted-printable, разные `multipart/alternative`/`multipart/related`/`multipart/encrypted` — крейт работает строго с `multipart/mixed` + `text/plain` + `application/json`;
- не валидирует email-адреса в полях `From`/`To` — это работа SMTP-слоя;
- не делает retry, throttling, deduplication писем — это ответственность `liveletters-sync`;
- не сохраняет ничего в БД — это ответственность `liveletters-store`.

## Граница с `liveletters-mail`

`liveletters-mime` ничего не знает про TCP, SMTP, IMAP, TlsStream, нативный TLS. Всё, что он умеет — превращать строки в строго типизированные структуры.

`liveletters-mail` (через `src/transport/`) поднимает на этом уровень выше:

- знает, что `OutgoingEmail.raw_message` надо отдать в SMTP `DATA`;
- знает, что из IMAP-ответа надо собрать `ReceivedEmail` и сразу прогнать через `parse_email` + `extract_liveletters_parts` + `decode_protocol_message`;
- знает, что `MimeError` нужно сконвертировать в `TransportError` через `From<MimeError> for TransportError`.

Граница сознательно оставлена односторонней: `liveletters-mail` зависит от `liveletters-mime`, но не наоборот.
