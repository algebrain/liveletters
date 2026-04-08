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

## Связанные документы

- `/home/algebrain/src/my/liveletters/docs/idea.technical.md`
- `/home/algebrain/src/my/liveletters/docs/workspace-structure.md`
