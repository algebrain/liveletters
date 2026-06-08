# liveletters-mime

## Назначение

`liveletters-mime` это библиотека разбора и сборки email-писем LiveLetters. Она отвечает за парсинг сырого RFC 5322-текста, извлечение человекочитаемой и технической частей, а также за сборку исходящего письма из `ProtocolMessage`.

## Зона ответственности

- разбор сырого текста письма на заголовки и тело;
- нормализация CRLF → LF;
- парсинг RFC 5322-заголовков (включая folded continuation lines) через `mailparse`;
- извлечение `human_readable_body` и `technical_body` из multipart-письма;
- сборка `OutgoingEmail` из `ProtocolMessage`;
- разбор `ProtocolMessage` из JSON-строки через `liveletters_protocol::decode_message` с человекочитаемой формулировкой ошибок;
- типизированные структуры `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts`.

## Что модуль не должен делать

- реализовывать SMTP/IMAP/TCP/TLS — это `liveletters-mail`;
- хранить письма или эвенты в SQLite — это `liveletters-store`;
- делать retry/throttling/dedup — это `liveletters-sync`;
- разбирать quoted-printable, base64, MIME-вложения; эту работу делает `mailparse`;
- валидировать email-адреса в `From`/`To`.

## Почему выделен в отдельный крейт

`liveletters-mime` существует отдельно от `liveletters-mail` по двум причинам.

**Тестируемость без сети.** MIME-логика нужна тестам даже тогда, когда `feature = "network"` в `liveletters-mail` отключена. Без отдельного креста пришлось бы либо тащить всю SMTP/IMAP-инфраструктуру в MIME-тесты, либо выдумывать in-memory fakes, что запрещено правилами проекта.

**Повторное использование в CLI и Tauri.** MIME-парсинг — это чистая функция от строки к структуре, без сетевых побочных эффектов. Эту функцию удобно дёргать из будущего `apps/lltt` и из Tauri-команд одинаковым образом.

Зависимости при этом минимальны: `liveletters-mime` зависит от `liveletters-protocol` и `mailparse`. Никаких знаний про `liveletters-store`, `liveletters-app-core`, `liveletters-config`. `mailparse` обрабатывает folded-заголовки, Content-Transfer-Encoding, charset-декодирование и предоставляет структурированный MIME-дерево (парсинг multipart-границ).

## Конвейер обработки входящего письма

Путь `raw_email` → `ProtocolMessage` идёт через три явных шага:

```
parse_email(raw_email)            → ParsedEmail
extract_liveletters_parts(parsed) → ExtractedMailParts
decode_protocol_message(parts.technical_body())
                                   → ProtocolMessage
```

Каждый шаг — отдельная функция с собственным `Result`-типом, и каждый шаг можно использовать по отдельности. Это разделение сделано сознательно:

- `parse_email` ничего не знает о LiveLetters-структуре: он работает с любым RFC 5322-письмом;
- `extract_liveletters_parts` знает про LiveLetters-формат, но не зависит от `liveletters_protocol`;
- `decode_protocol_message` инкапсулирует `liveletters_protocol::decode_message` с человекочитаемой формулировкой ошибок.

Такое разделение даёт два эффекта:

- `MimeError` гранулирован: «битый MIME» отдельно, «битый протокол» отдельно, «нет human-части» отдельно;
- тесты могут проверять каждый шаг конвейера отдельно, а не «всё или ничего».

## Формат LiveLetters-письма

Собранное `build_protocol_email` письмо имеет строго фиксированную форму:

```
From: <from>
To: <to>
Subject: <subject>
X-LiveLetters-Protocol: v1
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="liveletters-boundary"

--liveletters-boundary
Content-Type: text/plain; charset="utf-8"

<human_readable_body>
--liveletters-boundary
Content-Type: application/json; name="liveletters.json"
Content-Disposition: attachment; filename="liveletters.json"

<JSON-сериализованный ProtocolMessage>
--liveletters-boundary--
```

Заголовок `X-LiveLetters-Protocol: v1` нужен IMAP-слою: `lltt sync pull` ищет по нему входящие письма LiveLetters и не скачивает обычную почту целиком.

`Content-Type` письма — `multipart/mixed` с фиксированной границей `liveletters-boundary`. Человекочитаемая часть лежит в `text/plain; charset="utf-8"` под-части, а сериализованный JSON — в `application/json` под-части с `Content-Disposition: attachment; filename="liveletters.json"`. Никакого base64url-кодирования: JSON передаётся как есть в именованной MIME-части. Это снимает проблему с антиспам-фильтрами, которые ранее флагали длинный base64url-блок внутри `text/plain` как обфускацию.

Заголовок `Subject` кодируется через `encode_rfc2047`: если строка содержит не-ASCII символы (например, кириллицу), она оборачивается в `=?utf-8?B?<base64>?=`. Чистый ASCII передаётся без изменений. Это нужно, потому что RFC 5322 требует ASCII в заголовках — несоблюдение приводит к тому, что SMTP-серверы (например, Яндекса) удаляют не-ASCII заголовки из письма. `mailparse` на стороне получателя автоматически декодирует RFC 2047-строку обратно в исходный текст через `MailHeader::get_value()`, поэтому round-trip прозрачен.

## Парсинг MIME через `mailparse`

Вместо ручного разбора заголовков и MIME-границ `liveletters-mime` использует библиотеку [`mailparse`](https://crates.io/crates/mailparse). Она берёт на себя:

- парсинг RFC 5322-заголовков (включая continuation lines с пробелом/табуляцией);
- декодирование `Content-Transfer-Encoding` (quoted-printable, base64);
- charset-декодирование тела (`text/plain; charset=utf-8` → UTF-8);
- извлечение structured MIME-дерева (`ParsedMail.subparts`) из multipart-писем.

`parse_email` в `src/parser.rs` вызывает `mailparse::parse_mail` для разбора заголовков и сохраняет нормализованный сырой email в `ParsedEmail.raw`. Это позволяет `extract_liveletters_parts` в `src/mime.rs` повторно вызывать `mailparse::parse_mail` для навигации по MIME-дереву без хранения дополнительных структур.

Для `text/plain`-части используется `get_body()` (учёт charset), для `application/json` — `get_body_raw()` + `String::from_utf8` (JSON всегда UTF-8, без charset-преобразования).

## Нормализация CRLF → LF

`parse_email` первой строкой делает `raw_email.replace("\r\n", "\n")`. Нормализованный текст сохраняется в `ParsedEmail.raw`. Метод `body()` возвращает всё, что находится после первого `\n\n` (разделитель заголовков и тела). `mailparse::parse_mail` работает как с `\n`, так и с `\r\n` — нормализация нужна только для нашего `body()`.

Побочный эффект: если внутри тела письма есть символы `\r`, не привязанные к `\n`, они остаются как есть. Никакого дополнительного вычищения не производится.

## Формат ошибок

`MimeError`:

- `Protocol(String)` — сбой `liveletters_protocol`: `BlankEnvelopeField(field)` / `BlankHumanReadableBody` / `MalformedJson(message)`. Содержит человекочитаемую формулировку, а не Debug-строку.
- `InvalidEmailFormat(&'static str)` — структурная проблема MIME или ошибка `mailparse`. Сообщение фиксировано в коде, без пользовательских данных, чтобы не давать потенциальному атакующему лишних подсказок о содержимом письма.
- `MissingHumanReadablePart` — в multipart-теле не нашлась `text/plain` под-часть.
- `MissingTechnicalPart` — в multipart-теле не нашлась `application/json` под-часть.

`From<MimeError> for liveletters_mail::TransportError` объявлен в `liveletters-mail::lib.rs`. Это позволяет transport-слою возвращать `MimeError` наверх через единый `Result<_, TransportError>` без `map_err` на каждом шаге.

## Минимальный состав реализации

Модуль включает:

- `parse_email`, `extract_liveletters_parts`, `build_protocol_email`, `decode_protocol_message`;
- `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts`;
- `MimeError` с четырьмя вариантами;
- `crate_name() -> &'static str` для диагностики;
- integration-тесты в `tests/parse.rs` (7 тестов): разбор заголовков и тела, наличие `X-LiveLetters-Protocol`, извлечение human+technical, round-trip build→parse→extract→decode, отсутствие JSON-вложения, отказ на письме без `\n\n`, отказ на письме без multipart, письма с folded-заголовками;
- 1 lib-test `exposes_crate_name`.

## Требования к структуре каталога

- `src/lib.rs`;
- `src/build.rs`;
- `src/error.rs`;
- `src/message.rs`;
- `src/mime.rs`;
- `src/parser.rs`;
- `tests/parse.rs`.

Все файлы `src/` ≤ 63 строк. Лимит 600 строк на файл соблюдается с большим запасом.

## Требования к тестам

Покрытие тестами обязательно.

Реализованные проверки:

- `parses_headers_and_body_from_protocol_email` — `parse_email` корректно достаёт `Subject` и сохраняет тело письма;
- `parses_human_and_protocol_from_multipart_with_filename` — `extract_liveletters_parts` достаёт human+technical из multipart с `filename="liveletters.json"`;
- `build_and_decode_round_trip_preserves_payload` — round-trip `build → parse → extract → decode` сохраняет `DomainEventPayload::PostCreated { post_id, .. }`;
- `parse_email_rejects_message_without_blank_line_separator` — без `\n\n` между заголовками и телом возвращается `MimeError::InvalidEmailFormat`;
- `extract_liveletters_parts_rejects_non_multipart_email` — на `text/plain`-письме возвращается `MimeError::InvalidEmailFormat`;
- `parse_email_handles_folded_headers_without_crashing` — `parse_email` не падает на письме с folded-заголовками (Received, DKIM-Signature с continuation lines);
- `extract_parts_from_email_with_folded_headers` — `extract_liveletters_parts` извлекает human+technical из письма с folded-заголовками, `decode_protocol_message` корректно парсит payload.

Тесты намеренно идут через настоящие строки, а не через in-memory fakes. `sample_protocol_mime()` собирает «сырое» письмо через `build_protocol_email`, а потом прогоняет его через парсер. `folded-headers.eml` в `tests/fixtures/` — синтетическое письмо с folded-заголовками (Received из 3 цепочек, многострочная DKIM-Signature, Authentication-Results) и тестовыми адресами (@example.test).

## Требования к документации

Обязательна документация:

- описание конвейера `parse_email → extract_liveletters_parts → decode_protocol_message`;
- описание формата LiveLetters-письма;
- описание роли `build_protocol_email`;
- описание вариантов `MimeError`;
- явная фиксация того, что модуль не делает (произвольные вложения, quoted-printable, несколько альтернативных `multipart`-веток).

## Критерии готовности

- `parse_email` разбирает любой RFC 5322-текст с `\n\n` между заголовками и телом, включая письма с folded-заголовками (Received, DKIM-Signature с continuation lines);
- `extract_liveletters_parts` достаёт человекочитаемый текст и JSON-часть из multipart-письма;
- `build_protocol_email` собирает multipart-письмо с `text/plain` и `application/json` частями;
- round-trip `build → parse → extract → decode` сохраняет `ProtocolMessage` побитово;
- 7 integration-тестов и 1 lib-test зелёные.

Свойства реализации:

- выделение MIME-логики в отдельный крейт без зависимостей от SMTP/IMAP;
- типизированные структуры `OutgoingEmail`/`ReceivedEmail`/`ParsedEmail`/`ExtractedMailParts`;
- покрытие тестами round-trip и всех четырёх классов ошибок `MimeError`.

Ограничения:

- нет поддержки `multipart/alternative`/`multipart/related` — крейт работает только с `multipart/mixed`;
- нет строгой валидации email-адресов в `From`/`To` — это работа SMTP-слоя;
- ошибки `mailparse` маппятся в `MimeError::InvalidEmailFormat` со статической строкой; детализованное сообщение `MailParseError` не сохраняется (безопасность: не давать атакующему подсказок о содержимом письма).

Эти направления не входят в границы модуля.

## Связанные документы

- [idea.technical.md](../../docs/idea.technical.md)
- [technical-plan.md](../../docs/technical-plan.md)
- [liveletters-mail INTERFACE.md](../liveletters-mail/INTERFACE.md)
- [liveletters-mail TECHNICAL_SPEC.md](../liveletters-mail/TECHNICAL_SPEC.md)
