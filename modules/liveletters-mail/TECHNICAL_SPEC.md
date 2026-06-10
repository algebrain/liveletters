# liveletters-mail

## Назначение

`liveletters-mail` это библиотека почтового транспорта LiveLetters. Она отвечает за IMAP, SMTP, разбор email и низкоуровневое извлечение MIME-структуры.

## Зона ответственности

- SMTP send;
- IMAP fetch;
- mailbox scanning;
- raw email parsing;
- MIME extraction;
- transport-level retries;
- mapping raw email в внутренние транспортные типы.

## Что модуль не должен делать

- применять доменные события;
- решать вопросы авторизации;
- содержать UI-логику;
- знать о materialized views;
- быть местом orchestration сложных use cases.

## Основные подсистемы

- SMTP adapter;
- IMAP adapter;
- transport configuration;
- raw parser;
- MIME extractor;
- transport retry policy;
- mailbox cursor helpers.

## Требования к API

- получение новых писем;
- отправка письма;
- конфигурация реальных SMTP и IMAP подключений;
- парсинг сырого письма;
- извлечение технической части и человекочитаемой части;
- cursor-based fetch для почтового ящика;
- статусы send/fetch операций;
- транспортные ошибки как отдельные типы;
- абстракции для тестовых transport adapters.

## Требования к структуре каталога

- `src/imap`;
- `src/smtp`;
- `src/parser`;
- `src/mime`;
- `src/errors`;
- `tests/fixtures`;
- `tests`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на parsing email fixtures;
- tests на MIME edge cases;
- tests на transport-level retry behavior;
- tests на чтение multipart сообщений;
- tests на ошибки аутентификации и сетевые ошибки через adapters или mocks.
- tests на реальные TCP-seam adapters через локальные fixture-серверы.

## Требования к документации

Обязательна документация:

- описание транспортного API;
- поддерживаемые форматы email;
- правила обработки multipart писем;
- ограничения и негарантии transport слоя;
- способ настройки IMAP и SMTP.

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- `ConfiguredSmtpTransport` и `ConfiguredImapMailbox` для реального TCP transport seam под `#[cfg(feature = "network")]`;
- `SmtpTransportConfig`, `ImapMailboxConfig` и `MailAuth`;
- `MailboxCursor`, `FetchBatch`, `SendStatus`, `FetchStatus`;
- re-exports из `liveletters-mime`: `OutgoingEmail`, `ReceivedEmail`, `parse_email`, `extract_liveletters_parts`, `build_protocol_email`, `decode_protocol_message`, `MimeError`.

### Четырёхуровневый fallback для IMAP

`ConfiguredImapMailbox::fetch_new` использует четырёхуровневый
fallback, чтобы работать с IMAP-серверами разной степени
«строгости»:

1. **`UID SEARCH UID <n>:* HEADER X-LiveLetters-Protocol v1`** —
   самый быстрый и экономный по трафику способ. Поддерживается
   большинством современных серверов (gmail, fastmail). Некоторые
   серверы (mail.ru) отвечают `NO [CANNOT] Unsupported search
   criterion`.
2. **`UID SEARCH UID <n>:*` + `UID FETCH <uid> BODY.PEEK[HEADER.FIELDS (X-LiveLetters-Protocol)]`** —
   список всех UID, затем для каждого — только нужный заголовок.
   Совместимо с большинством серверов. Некоторые (mail.ru) отвечают
   `BAD [PARSE]` на синтаксис `HEADER.FIELDS`.
3. **`UID FETCH <uid> BODY.PEEK[HEADER]`** — все заголовки целиком,
   клиент сам ищет в них нужный. Совместимо со всеми серверами,
   поддерживающими IMAP4rev1.
4. **`UID FETCH <uid> BODY.PEEK[]` + локальный парсинг заголовков** —
   всё тело письма. Самый дорогой по трафику, но работает даже на
   самых ограниченных серверах.

Клиент переходит на следующий уровень только если предыдущий
вернул `BAD` или `PARSE` от сервера. В обычной ситуации (yandex.ru,
gmail, fastmail) достаточно первого или второго уровня. На mail.ru
срабатывает третий. Самый экзотический сервер, отвергающий и
`BODY.PEEK[HEADER]`, обслуживается четвёртым.

### Вспомогательные методы IMAP

- `find_min_uid_since_days(days)` — открывает отдельное IMAP-соединение,
  выполняет `UID SEARCH SINCE <дата>`, возвращает минимальный UID
  среди писем за последние `days` суток. Используется при первом
  запуске с `initial_lookback_days` и при backfill.
- `anchor_for_backfill(days)` — обёртка над `find_min_uid_since_days`,
  возвращает готовый `MailboxCursor::start_with_since_uid(since_uid)`
  с `since_uid = max(1, min_uid)`.

Дата для `UID SEARCH SINCE` вычисляется в формате IMAP
`DD-Mon-YYYY` (например, `09-Jun-2026`) без внешних зависимостей,
через `since_date_for_today_minus(days)`.

Тестовые сценарии для transport слоя строятся без in-memory подделок: SMTP и IMAP проверяются через `tests/network_flow.rs`, где поднимается локальный `TcpListener` и проверяется честный TCP-обмен, включая байтовое чтение IMAP literal, поиск `X-LiveLetters-Protocol: v1` через `SEARCH HEADER`, fallback через `HEADER.FIELDS`, fallback через `BODY.PEEK[HEADER]` (mail.ru-сценарий) и fallback через `BODY.PEEK[]` (экзотический сервер). MIME-уровень проверяется на уровне `liveletters-mime` через round-trip `build_protocol_email` + `decode_protocol_message` и разбор заранее подготовленных raw-писем.

Текущие реальные adapters пока ориентированы на plaintext TCP seam без TLS и нужны как честная интеграционная база для следующего прохода, а не как завершенный production transport.

## Критерии готовности

- библиотека умеет отправлять и получать email через абстракции;
- библиотека умеет отправлять и получать email через конфигурируемый TCP seam;
- raw parsing работает на fixture-наборе;
- ошибки транспорта типизированы;
- библиотека не принимает доменные решения.

## Связанные документы

- [idea.technical.md](../../docs/idea.technical.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
