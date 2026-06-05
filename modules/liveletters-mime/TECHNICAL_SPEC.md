# liveletters-mime

## Назначение

`liveletters-mime` это библиотека разбора и сборки email-писем LiveLetters в их MIME-форме. Она отвечает за парсинг сырого RFC 5322-текста, извлечение из multipart-тела человекочитаемой и технической частей, а также за сборку исходящего письма из `ProtocolMessage`.

## Зона ответственности

- разбор сырого текста письма на заголовки и тело;
- нормализация CRLF → LF;
- извлечение `human_readable_body` (`text/plain`) и `technical_body` (`application/json`) из multipart/mixed-тела;
- сборка `OutgoingEmail` из `ProtocolMessage` с фиксированной границей `liveletters-boundary`;
- разбор `ProtocolMessage` из JSON-строки через `liveletters_protocol::decode_message` с человекочитаемой формулировкой ошибок;
- типизированные структуры `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts`.

## Что модуль не должен делать

- реализовывать SMTP/IMAP/TCP/TLS — это `liveletters-mail`;
- хранить письма или эвенты в SQLite — это `liveletters-store`;
- делать retry/throttling/dedup — это `liveletters-sync`;
- разбирать base64/quoted-printable и MIME-вложения — текущая версия работает только с `text/plain` + `application/json`;
- валидировать email-адреса в `From`/`To`.

## Почему выделен в отдельный крейт

`liveletters-mime` существует отдельно от `liveletters-mail` по двум причинам.

**Тестируемость без сети.** MIME-логика нужна тестам даже тогда, когда `feature = "network"` в `liveletters-mail` отключена. Без отдельного креста пришлось бы либо тащить всю SMTP/IMAP-инфраструктуру в MIME-тесты, либо выдумывать in-memory fakes, что запрещено правилами проекта.

**Повторное использование в CLI и Tauri.** MIME-парсинг — это чистая функция от строки к структуре, без сетевых побочных эффектов. Эту функцию удобно дёргать из будущего `apps/lltt` и из Tauri-команд одинаковым образом.

Зависимости при этом минимальны: `liveletters-mime` зависит только от `liveletters-protocol`. Никаких знаний про `liveletters-store`, `liveletters-app-core`, `liveletters-config`.

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
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="liveletters-boundary"

--liveletters-boundary
Content-Type: text/plain; charset="utf-8"

<human_readable_body>
--liveletters-boundary
Content-Type: application/json

<JSON-сериализованный ProtocolMessage>
--liveletters-boundary--
```

Граница `liveletters-boundary` намеренно детерминирована и зашита как константа в `build.rs`. Это не цитата из RFC 2046, а собственный маркер LiveLetters. Его детерминированность нужна для воспроизводимости тестов round-trip.

`Content-Type` верхнего уровня — `multipart/mixed`. `Content-Type` каждой части — `text/plain; charset="utf-8"` и `application/json` соответственно. Никаких вложений, альтернативных версий, related-частей текущая версия не поддерживает.

## Парсинг `Content-Type` и `boundary`

`extract_boundary` в `src/mime.rs` парсит `Content-Type` очень простым способом:

- ищет подстроку `boundary=`;
- берёт всё после неё;
- триммит пробелы и снимает внешние кавычки.

Это покрывает канонические случаи `boundary="liveletters-boundary"` и `boundary=liveletters-boundary`, но не покрывает:

- `boundary` с пробелами внутри значения (RFC 2046 не разрешает, но некоторые кривые MTA могут генерировать);
- `boundary*0*` / `boundary*1*` (RFC 2231 encoded continuation);
- экранированные кавычки внутри значения.

Эти краевые случаи считаются «битым MIME» и приводят к `MimeError::InvalidEmailFormat`. Для LiveLetters это приемлемо, потому что отправитель — это тот же код, что и получатель, и несовместимые MTA не используются.

## Нормализация CRLF → LF

`parse_email` первой строкой делает `raw_email.replace("\r\n", "\n")`. Это означает, что в `body()` всегда лежит текст с LF-окончаниями строк, и весь дальнейший разбор (`\n\n` как разделитель заголовков и тела, `header_block.lines()`) работает в одной нотации.

Побочный эффект: если внутри тела письма есть символы `\r`, не привязанные к `\n`, они остаются как есть. Текущая реализация не пытается их вычищать.

## Формат ошибок

`MimeError`:

- `Protocol(String)` — сбой `liveletters_protocol`: `BlankEnvelopeField(field)` / `BlankHumanReadableBody` / `MalformedJson(message)`. Содержит человекочитаемую формулировку, а не Debug-строку.
- `InvalidEmailFormat(&'static str)` — структурная проблема MIME. Сообщение фиксировано в коде, без пользовательских данных, чтобы не давать потенциальному атакующему лишних подсказок о содержимом письма.
- `MissingHumanReadablePart` — в multipart-теле не нашлось `text/plain`.
- `MissingTechnicalPart` — в multipart-теле не нашлось `application/json`.

`From<MimeError> for liveletters_mail::TransportError` объявлен в `liveletters-mail::lib.rs`. Это позволяет transport-слою возвращать `MimeError` наверх через единый `Result<_, TransportError>` без `map_err` на каждом шаге.

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- `parse_email`, `extract_liveletters_parts`, `build_protocol_email`, `decode_protocol_message`;
- `OutgoingEmail`, `ReceivedEmail`, `ParsedEmail`, `ExtractedMailParts`;
- `MimeError` с четырьмя вариантами;
- `crate_name() -> &'static str` для диагностики;
- 5 integration-тестов в `tests/parse.rs`: разбор заголовков и тела, извлечение human+technical, round-trip build→parse→extract→decode, отказ на письме без `\n\n`, отказ на non-multipart-письме;
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

- `parses_headers_and_body_from_multipart_email` — `parse_email` корректно достаёт `Subject` и сохраняет тело с маркерами границ;
- `extracts_human_and_technical_parts_from_multipart_email` — `extract_liveletters_parts` достаёт и человекочитаемую, и техническую часть;
- `build_and_decode_round_trip_preserves_payload` — round-trip `build → parse → extract → decode` сохраняет `DomainEventPayload::PostCreated { post_id, .. }`;
- `parse_email_rejects_message_without_blank_line_separator` — без `\n\n` между заголовками и телом возвращается `MimeError::InvalidEmailFormat`;
- `extract_liveletters_parts_rejects_non_multipart_email` — на `text/plain`-письме без multipart возвращается `MimeError::InvalidEmailFormat`.

Тесты намеренно идут через настоящие строки, а не через in-memory fakes. `sample_multipart_mime()` собирает «сырое» письмо через `build_protocol_email`, а потом прогоняет его через парсер. Это закрывает как минимум round-trip баг класса «изменили формат, забыли обновить парсер».

## Требования к документации

Обязательна документация:

- описание конвейера `parse_email → extract_liveletters_parts → decode_protocol_message`;
- описание формата LiveLetters-письма с фиксированной границей;
- описание роли `build_protocol_email` и детерминированности `liveletters-boundary`;
- описание вариантов `MimeError`;
- явная фиксация того, что модуль не делает (вложения, base64, quoted-printable).

## Критерии готовности

- `parse_email` разбирает любой RFC 5322-текст с `\n\n` между заголовками и телом;
- `extract_liveletters_parts` достаёт `text/plain` и `application/json` из `multipart/mixed` с параметром `boundary=`;
- `build_protocol_email` собирает письмо с фиксированной границей `liveletters-boundary`;
- round-trip `build → parse → extract → decode` сохраняет `ProtocolMessage` побитово;
- 5 integration-тестов и 1 lib-test зелёные.

Сейчас практически считаются уже закрытыми:

- выделение MIME-логики в отдельный крейт без зависимостей от SMTP/IMAP;
- фиксированная граница multipart;
- типизированные структуры `OutgoingEmail`/`ReceivedEmail`/`ParsedEmail`/`ExtractedMailParts`;
- покрытие тестами round-trip и всех четырёх классов ошибок `MimeError`.

Модуль пока не считается завершенным в части:

- нет поддержки base64-encoded body;
- нет поддержки quoted-printable;
- нет поддержки вложений и `multipart/alternative`/`multipart/related`;
- нет нормализации `Content-Transfer-Encoding`;
- нет строгой валидации email-адресов в `From`/`To`.

Эти направления зафиксированы как возможный следующий шаг, но не блокируют текущую версию.

## Связанные документы

- [idea.technical.md](../../docs/idea.technical.md)
- [technical-plan.md](../../docs/technical-plan.md)
- [liveletters-mail INTERFACE.md](../liveletters-mail/INTERFACE.md)
- [liveletters-mail TECHNICAL_SPEC.md](../liveletters-mail/TECHNICAL_SPEC.md)
