# liveletters-sync

## Назначение

`liveletters-sync` это библиотека синхронизации и репликации LiveLetters. Она преобразует входящие протокольные сообщения в валидированные доменные изменения локального состояния.

## Зона ответственности

- ingest входящих сообщений;
- классификация писем;
- выделение LiveLetters-сообщений;
- извлечение доменных событий;
- валидация структуры;
- дедупликация;
- anti-replay;
- применение доменных событий;
- обработка отложенных зависимостей;
- интеграция с журналом ошибок ingest.

## Что модуль не должен делать

- напрямую управлять UI;
- быть Tauri boundary;
- содержать низкоуровневую реализацию IMAP/SMTP;
- описывать MIME-формат вместо `liveletters-protocol`.

## Основные входные зависимости

- `liveletters-domain`;
- `liveletters-protocol`;
- `liveletters-store`;
- `liveletters-mail`.

## Основные сценарии

- импорт входящих писем из inbox;
- распознавание valid event;
- обработка duplicate event;
- обработка replayed event;
- обработка unauthorized event;
- обработка malformed event;
- откладывание событий без зависимостей;
- повторная переобработка отложенных событий.

## Требования к API

- ingest batch;
- classify message;
- extract domain events;
- validate event;
- apply event transactionally;
- schedule deferred processing;
- report sync outcome.

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- ingest batch для входящих писем;
- parsing через `liveletters-mail`;
- decode `ProtocolMessage`;
- базовую валидацию envelope/payload consistency;
- разделение исходов:
  - `Applied`
  - `Duplicate`
  - `Replay`
  - `Unauthorized`
  - `Invalid`
  - `Deferred`
  - `Malformed`
- запись `raw_messages` и `raw_events` в `liveletters-store`;
- apply-status и failure reason для `raw_events`;
- переобработку `deferred_events` после появления зависимостей.

Текущая authorization policy пока минимальна и intentionally conservative:

- `PostHidden` разрешается только автору исходного поста;
- `CommentEdited` разрешается только автору исходного комментария.

Текущая anti-replay policy тоже минимальна:

- duplicate определяется по уже известному `event_id`;
- replay определяется по повторному воспроизведению уже материализованного состояния с новым `event_id`.

## Требования к структуре каталога

- `src/ingest`;
- `src/classify`;
- `src/validate`;
- `src/dedup`;
- `src/apply`;
- `src/deferred`;
- `src/reporting`;
- `tests/fixtures`;
- `tests`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на out-of-order delivery;
- tests на duplicate delivery;
- tests на replay cases;
- tests на malformed messages;
- tests на unauthorized events;
- tests на deferred dependency resolution;
- integration tests на применение batch сообщений.

## Требования к документации

Обязательна документация:

- pipeline ingest;
- классификация входящих писем;
- anti-replay policy;
- deferred event policy;
- правила применения и отклонения событий;
- структура sync reports.

## Критерии готовности

- библиотека стабильно обрабатывает основные классы входящих писем;
- out-of-order и duplicate cases покрыты тестами;
- pipeline документирован;
- sync logic отделен от transport и UI.

Для текущего этапа второго прохода practically считаются уже закрытыми:

- `duplicate` vs `replay` как разные исходы;
- `unauthorized` и `invalid` как отдельные исходы;
- deferred reprocessing;
- richer sync outcome для следующего слоя.

Но модуль еще не считается окончательно завершенным:

- authorization policy пока минимальна;
- apply-status хранится в простой строковой форме;
- ingest pipeline еще не связан с более богатым diagnostics и backend boundary.

## Связанные документы

- [idea.technical.md](../../docs/idea.technical.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
