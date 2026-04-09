# `liveletters-mail` INTERFACE

## Назначение

`liveletters-mail` это внешний программный интерфейс почтового транспортного слоя LiveLetters.

Если смотреть на уже пройденные модули по порядку:

- `liveletters-domain` задает форму доменных фактов;
- `liveletters-protocol` задает форму технического сообщения;
- `liveletters-store` хранит локальное состояние и технические журналы;
- `liveletters-app-core` собирает прикладные сценарии;
- `liveletters-mail` отвечает за следующий уровень: как техническое сообщение живет внутри email и как его доставлять или читать через почтовый seam.

Этот модуль не принимает доменных решений. Его задача уже:

- собрать email из protocol message;
- отправить email;
- получить email;
- распарсить сырое письмо;
- извлечь из него человекочитаемую и техническую части;
- дать верхним слоям типизированные транспортные результаты и ошибки.

Важно различать:

- `liveletters-protocol` знает формат технического payload;
- `liveletters-mail` знает, как этот payload упаковать и достать из email.

## Где находится интерфейс

- crate: `liveletters-mail`
- точка подключения: `src/lib.rs`

Наружу модуль экспортирует:

- транспортные конфигурации;
- транспортные модели писем;
- SMTP и IMAP adapters;
- cursor/status типы;
- parsing и MIME helpers;
- retry policy;
- `TransportError`.

## Что считается внешним интерфейсом этого модуля

С практической точки зрения внешний интерфейс `liveletters-mail` это:

1. типы конфигурации IMAP и SMTP;
2. типы `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts`;
3. SMTP и IMAP adapters, как in-memory, так и configured;
4. функции `build_protocol_email(...)`, `parse_email(...)`, `extract_liveletters_parts(...)`, `decode_protocol_message(...)`;
5. курсоры и статусы fetch/send;
6. `MailRetryPolicy`;
7. `TransportError`.

Именно этим API пользуются:

- `liveletters-sync`;
- прикладные интеграционные тесты;
- любой верхний слой, которому нужен транспортный seam без прямой работы с сокетами и MIME-деталями.

## Главная идея этого модуля

На уровне внешнего использования `liveletters-mail` решает три разные задачи.

### 1. Сборка исходящего письма

Нужно взять `ProtocolMessage` и превратить его в email, который:

- имеет привычные заголовки;
- содержит человекочитаемую часть;
- содержит технический JSON payload.

### 2. Чтение входящего письма

Нужно получить raw email и представить его в более удобной транспортной форме.

### 3. Извлечение LiveLetters-содержимого

Нужно из распарсенного письма достать:

- human-readable часть;
- технический JSON payload.

То есть это модуль не про “почтовый клиент целиком”, а про честный transport seam вокруг LiveLetters-сообщений.

## Конфигурация транспорта

Наружу экспортируются:

- `MailAuth`
- `SmtpTransportConfig`
- `ImapMailboxConfig`

## `MailAuth`

### Зачем нужен этот тип

`MailAuth` описывает способ аутентификации на транспортном уровне.

Сейчас он поддерживает два режима:

- `None`
- `Password { username, password }`

Смысл такого интерфейса:

- верхний слой явно выбирает, есть ли аутентификация;
- transport adapters не принимают абстрактную неструктурированную “конфигурацию логина”;
- форма данных фиксируется типом.

Важно понимать ограничение:

- текущий модуль пока не реализует богатую матрицу auth-механизмов;
- это минимальный, но реальный seam текущей реализации.

## `SmtpTransportConfig`

### Зачем нужен этот тип

Это конфигурация исходящего SMTP transport.

Он хранит:

- `server`
- `port`
- `hello_domain`
- `auth`

Смысл полей:

- `server` и `port` описывают, куда подключаться;
- `hello_domain` нужен для SMTP greeting;
- `auth` задает политику аутентификации.

### Как его использовать

Создается через:

- `SmtpTransportConfig::new(server, port, hello_domain, auth)`

Затем его обычно передают в:

- `ConfiguredSmtpTransport::new(config)`

Читать обратно можно через:

- `server()`
- `port()`
- `hello_domain()`
- `auth()`

## `ImapMailboxConfig`

### Зачем нужен этот тип

Это конфигурация входящего IMAP mailbox seam.

Он хранит:

- `server`
- `port`
- `mailbox`
- `auth`

Смысл полей:

- `server` и `port` определяют endpoint;
- `mailbox` определяет, какую почтовую папку читать;
- `auth` задает режим аутентификации.

### Как его использовать

Создается через:

- `ImapMailboxConfig::new(server, port, mailbox, auth)`

Обычно затем передается в:

- `ConfiguredImapMailbox::new(config)`

Чтение идет через:

- `server()`
- `port()`
- `mailbox()`
- `auth()`

## Транспортные модели писем

Наружу экспортируются:

- `OutgoingEmail`
- `ReceivedEmail`
- `ParsedEmail`
- `ExtractedMailParts`

Эти типы представляют разные стадии жизни письма.

## `OutgoingEmail`

### Что это такое

Это уже подготовленное исходящее письмо.

Оно хранит:

- `from`
- `to`
- `subject`
- `raw_message`

То есть это не абстрактное намерение “отправить сообщение”, а конкретный transport artifact, который можно отдать SMTP adapter.

### Когда используется

Обычно сценарий такой:

1. верхний слой собирает `ProtocolMessage`;
2. вызывает `build_protocol_email(...)`;
3. получает `OutgoingEmail`;
4. передает его в SMTP transport.

## `ReceivedEmail`

### Что это такое

Это сырое входящее письмо, уже полученное транспортным слоем.

Оно хранит:

- `message_id`
- `raw_message`

Смысл этого типа:

- transport уже получил письмо;
- теперь его можно отдавать дальше на parsing и sync.

Этот тип специально минимальный: он не делает вид, что письмо уже понято, он только фиксирует факт получения.

## `ParsedEmail`

### Зачем нужен этот тип

`ParsedEmail` это промежуточная форма между сырой строкой и извлечением полезных частей.

Он нужен затем, чтобы:

- разобрать заголовки;
- отделить body;
- дать удобный API чтения header-значений.

### Что с ним можно делать

Создается через:

- `ParsedEmail::new(headers, body)`

Читается через:

- `body()`
- `header(name)`
- `subject()`

Этот тип нужен верхним слоям и helper-функциям, которые хотят работать уже не со “всем письмом одной строкой”, но еще не с полноценным MIME-tree.

## `ExtractedMailParts`

### Зачем нужен этот тип

Это результат извлечения полезных частей LiveLetters-сообщения из письма.

Он хранит:

- `human_readable_body`
- `technical_body`

То есть это именно та форма, которая нужна дальше sync-контуру:

- человеческий слой можно показать или логировать;
- технический слой можно декодировать через `liveletters-protocol`.

### Что с ним можно делать

Создается через:

- `ExtractedMailParts::new(human_readable_body, technical_body)`

Читается через:

- `human_readable_body()`
- `technical_body()`

## Сборка исходящего письма: `build_protocol_email(...)`

### Что делает эта функция

`build_protocol_email(from, to, subject, protocol_message) -> Result<OutgoingEmail, TransportError>`

Она берет `ProtocolMessage` и упаковывает его в multipart email.

На текущем этапе ее поведение такое:

- создается обычный email с `From`, `To`, `Subject`, `MIME-Version`;
- письмо получает multipart boundary;
- человекочитаемая часть кладется как `text/plain`;
- техническая часть кладется как `application/json`.

### Зачем она нужна

Чтобы верхний слой не собирал MIME-письмо вручную.

То есть:

- `app-core` и другие верхние слои работают с `ProtocolMessage`;
- `liveletters-mail` берет на себя упаковку в форму, пригодную для почтовой отправки.

### Что она не делает

- не отправляет письмо сама;
- не открывает SMTP-соединение;
- не проверяет прикладную валидность события.

Она только превращает protocol message в `OutgoingEmail`.

## Декодирование технической части: `decode_protocol_message(...)`

### Что делает эта функция

`decode_protocol_message(input) -> Result<ProtocolMessage, TransportError>`

Это transport-friendly обертка над `liveletters-protocol::decode_message(...)`.

Ее смысл:

- верхний слой может оставаться внутри transport error boundary;
- ошибки протокольного декодирования маппятся в `TransportError::Protocol(...)`.

Это полезно для sync-слоя, который работает на стыке:

- email parsing;
- MIME extraction;
- protocol decoding.

## Разбор сырого письма: `parse_email(...)`

### Что делает эта функция

`parse_email(raw_email) -> Result<ParsedEmail, TransportError>`

Она:

- нормализует переводы строк;
- отделяет блок заголовков от тела;
- разбирает строки заголовков в пары имя/значение;
- строит `ParsedEmail`.

### Когда использовать

Это первый шаг после получения `ReceivedEmail`, если нужно понять письмо дальше.

Типичный сценарий:

1. пришел `ReceivedEmail`;
2. вызывается `parse_email(...)`;
3. затем вызывается `extract_liveletters_parts(...)`.

### Ограничения

Парсер сейчас намеренно простой:

- он ожидает обычную схему `headers + пустая строка + body`;
- он не реализует полный RFC-парсер почты.

Это важно понимать как ограничение transport seam текущей реализации.

## Извлечение полезных частей: `extract_liveletters_parts(...)`

### Что делает эта функция

`extract_liveletters_parts(parsed) -> Result<ExtractedMailParts, TransportError>`

Она:

- проверяет, что письмо multipart;
- находит boundary;
- проходит по частям письма;
- ищет `text/plain`;
- ищет `application/json`;
- возвращает их как `ExtractedMailParts`.

### Зачем это нужно

Это ключевой мост между email-формой и protocol-формой.

После этой функции верхний слой уже может:

- декодировать `technical_body` как `ProtocolMessage`;
- не разбирать MIME вручную.

### Что важно

Если в письме нет одной из ожидаемых частей, это считается transport-level проблемой, а не доменной ошибкой.

## SMTP adapters: как устроен исходящий seam

Наружу экспортируются:

- `InMemorySmtpTransport`
- `ConfiguredSmtpTransport`

## `InMemorySmtpTransport`

### Зачем нужен

Это тестовый и локальный transport adapter.

Он не ходит в сеть. Вместо этого он просто хранит отправленные письма в памяти.

Это полезно:

- для unit и integration tests;
- для проверки, что верхний слой действительно сформировал и “отправил” письмо;
- для сценариев, где нужна честная transport boundary, но не нужен реальный SMTP.

### Что с ним можно делать

- `new()`
- `send(email)`
- `sent_emails()`

То есть можно:

- отправить письмо;
- потом проверить, что именно было отправлено.

## `ConfiguredSmtpTransport`

### Зачем нужен

Это реальный TCP-based SMTP seam текущей реализации.

Он не делает вид, что это production-grade полноценный mail stack. Его смысл:

- дать честный интеграционный слой поверх конфигурации;
- уметь реально соединиться, пройти greeting, auth и отправку данных;
- не держать эту низкоуровневую логику во внешнем коде.

### Как его использовать

1. собрать `SmtpTransportConfig`;
2. создать `ConfiguredSmtpTransport::new(config)`;
3. вызвать `send(&OutgoingEmail)`.

### Что он возвращает

При успехе:

- `SendStatus::Sent`

При ошибке:

- `TransportError`

### Ограничения

Текущая реализация:

- ориентирована на простой TCP seam;
- не является полноценной production SMTP-библиотекой;
- должна восприниматься как честный минимальный transport layer, а не завершенный mail stack.

## IMAP adapters: как устроен входящий seam

Наружу экспортируются:

- `InMemoryImapMailbox`
- `ConfiguredImapMailbox`

## `InMemoryImapMailbox`

### Зачем нужен

Это тестовый входящий mailbox adapter.

Он нужен затем, чтобы:

- руками класть сырые письма в очередь;
- потом читать их как будто они пришли из почтового ящика.

### Что с ним можно делать

- `new()`
- `push_raw_email(message_id, raw_message)`
- `fetch_new()`
- `fetch_batch(cursor)`

То есть это тестовый источник `ReceivedEmail`.

## `ConfiguredImapMailbox`

### Зачем нужен

Это реальный TCP-based IMAP seam текущей реализации.

Он нужен, когда нужно честно получить письма из mailbox через конфигурацию, не реализуя протокол вручную во внешнем коде.

### Как его использовать

1. собрать `ImapMailboxConfig`;
2. создать `ConfiguredImapMailbox::new(config)`;
3. держать `MailboxCursor`;
4. вызывать `fetch_new(&cursor)`.

### Что он возвращает

Возвращается `FetchBatch`, внутри которого есть:

- новые `ReceivedEmail`;
- следующий cursor;
- статус fetch-операции.

### Ограничения

Как и SMTP seam, этот IMAP adapter сейчас:

- минимальный;
- ориентирован на реальный интеграционный контур;
- не претендует на полноту всех возможностей production IMAP-клиента.

## Курсоры и статусы: зачем они нужны

Наружу экспортируются:

- `SendStatus`
- `FetchStatus`
- `MailboxCursor`
- `FetchBatch`

## `SendStatus`

Сейчас это простой тип со значением:

- `Sent`

Его смысл:

- явно зафиксировать успешный результат send-операции;
- не возвращать “успех” неструктурированно.

## `FetchStatus`

Этот тип описывает результат чтения mailbox:

- `Fetched { message_count }`
- `NoNewMessages`

Он нужен затем, чтобы верхний слой понимал:

- действительно ли были новые письма;
- или операция прошла успешно, но ничего нового не нашлось.

## `MailboxCursor`

### Зачем нужен

Это тип для cursor-based чтения mailbox.

Его смысл:

- не читать ящик каждый раз “с нуля”;
- помнить, до какого места уже дочитали.

### Что с ним можно делать

- `start()`
- `from_last_seen_uid(...)`
- `last_seen_uid()`
- `advance_to(uid)`

То есть это минимальный внешний контракт инкрементального чтения mailbox.

## `FetchBatch`

### Зачем нужен

Это результат одной fetch-операции.

Он хранит:

- список писем;
- следующий cursor;
- статус операции.

### Что с ним можно делать

- `emails()`
- `into_emails()`
- `next_cursor()`
- `status()`

Это дает верхнему слою готовую форму для цикла:

1. получил batch;
2. обработал письма;
3. сохранил следующий cursor;
4. пошел дальше.

## `MailRetryPolicy`: зачем нужен этот тип

`MailRetryPolicy` это маленький helper для правил повторной попытки.

Он нужен, чтобы transport-поведение было не “если что, как-нибудь попробуем еще раз”, а выражалось явной политикой.

Сейчас с ним можно:

- создать политику через `new(max_attempts)`;
- спросить `should_retry(&error)`;
- проверить `allows_attempt(attempt_number)`.

На текущем этапе retry логика консервативная:

- retry допускается для `Network`;
- retry допускается для `UnexpectedResponse`;
- остальные ошибки считаются не тем классом проблем, который нужно автоматически повторять.

## `TransportError`: что означает эта ошибка

`TransportError` это единый внешний тип ошибок транспортного слоя.

Сейчас он содержит:

- `AuthenticationFailed`
- `Network(String)`
- `InvalidEmailFormat(&'static str)`
- `MissingHumanReadablePart`
- `MissingTechnicalPart`
- `Protocol(String)`
- `UnexpectedResponse(String)`
- `UnsupportedAuthMechanism(&'static str)`

## Как понимать эти ошибки

### `AuthenticationFailed`

Означает, что соединение с почтовым сервисом дошло до auth-фазы, но аутентификация не прошла.

### `Network(...)`

Означает сетевую проблему:

- не удалось подключиться;
- сокетная операция завершилась ошибкой;
- транспорт недоступен.

### `InvalidEmailFormat(...)`

Означает, что сырое письмо не имеет минимально допустимой формы для текущего parser/MIME contour.

### `MissingHumanReadablePart`

Означает, что письмо похоже на multipart, но в нем не удалось найти ожидаемую человекочитаемую часть.

### `MissingTechnicalPart`

Означает, что письмо не содержит нужный technical JSON part.

### `Protocol(...)`

Означает, что email уже был разобран до технической части, но сама техническая часть не прошла протокольное декодирование.

То есть это ошибка на стыке transport и protocol boundary.

### `UnexpectedResponse(...)`

Означает, что внешний почтовый сервер ответил чем-то, что текущий seam не считает ожидаемым.

### `UnsupportedAuthMechanism(...)`

Означает, что верхний слой попросил auth-механику, которую текущая реализация транспортного seam не поддерживает.

## Как верхние слои должны использовать этот модуль

Правильные сценарии использования обычно такие.

### Для исходящего контура

1. верхний слой собирает `ProtocolMessage`;
2. вызывает `build_protocol_email(...)`;
3. получает `OutgoingEmail`;
4. передает его в SMTP adapter;
5. получает `SendStatus` или `TransportError`.

### Для входящего контура

1. mailbox adapter возвращает `ReceivedEmail`;
2. вызывается `parse_email(...)`;
3. вызывается `extract_liveletters_parts(...)`;
4. `technical_body` декодируется через `decode_protocol_message(...)`;
5. дальше верхний слой передает сообщение в sync/apply-контур.

То есть `liveletters-mail` должен быть местом, где верхние слои перестают работать с сокетами и сырыми MIME-строками напрямую.

## Что этот модуль специально не делает

Чтобы не было ложных ожиданий, важно зафиксировать границы.

Этот модуль сейчас не делает следующего:

- не применяет доменные события;
- не решает вопросы duplicate, replay или authorization;
- не хранит сообщения в локальной базе;
- не считает sync health;
- не строит UI-friendly DTO;
- не является полноценным universal mail client stack.

То есть это transport boundary вокруг LiveLetters-сообщений, а не полный почтовый или sync-движок.

## Что считать стабильным контрактом здесь

Если смотреть глазами потребителя crate, то наиболее значимыми частями интерфейса являются:

1. формы transport config типов;
2. формы `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts`;
3. поведение `build_protocol_email(...)`, `parse_email(...)`, `extract_liveletters_parts(...)`, `decode_protocol_message(...)`;
4. public API SMTP/IMAP adapters;
5. `MailboxCursor`, `FetchBatch`, `FetchStatus`, `SendStatus`;
6. `MailRetryPolicy`;
7. `TransportError`.

Изменение этих элементов почти наверняка требует согласованных правок в:

- `liveletters-sync`;
- интеграционных transport-тестах;
- mail-related частях backend orchestration.

## Связанные документы

- [TECHNICAL_SPEC.md](./TECHNICAL_SPEC.md)
