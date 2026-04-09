# liveletters-store

## Назначение

`liveletters-store` это библиотека локального хранения данных LiveLetters. Она отвечает за SQLite-схему, миграции, репозитории и materialized state.

## Зона ответственности

- schema management;
- migrations;
- repositories;
- inbox journal;
- outbox journal;
- raw messages;
- raw events;
- materialized views;
- sync cursors;
- transaction boundaries.

## Что модуль не должен делать

- содержать UI DTO;
- реализовывать IMAP/SMTP;
- реализовывать Tauri glue code;
- хранить доменную бизнес-логику, не связанную с persistence.

## Основные таблицы

- `accounts`;
- `resources`;
- `posts`;
- `comments`;
- `subscriptions`;
- `memberships`;
- `raw_messages`;
- `raw_events`;
- `outbox_messages`;
- `sync_cursors`;
- `failed_ingest`;
- `feed_items`;
- `thread_views`.

## Требования к API

- явные repository contracts;
- транзакционные операции для критичных сценариев;
- функции для чтения materialized state;
- операции для записи raw messages и raw events;
- миграции как часть библиотеки;
- безопасная и тестируемая инициализация БД.

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- in-memory и file-based SQLite opening;
- базовые таблицы для:
  - `posts`
  - `comments`
  - `outbox`
  - `raw_messages`
  - `raw_events`
  - `deferred_events`
- repository-like операции для чтения и записи этих сущностей;
- home-scoped путь хранения по умолчанию;
- тестовый сценарий с временным `HOME`.

Текущий `raw_events` journal уже хранит не только payload, но и apply-related metadata:

- `apply_status`;
- `failure_reason`.

Текущий `deferred_events` contour уже поддерживает:

- сохранение отложенного события;
- чтение очереди отложенных событий;
- удаление события после успешной или окончательной переобработки.

## Требования к структуре каталога

- `src/db`;
- `src/migrations`;
- `src/repositories`;
- `src/models`;
- `src/materialized_views`;
- `tests`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на применение миграций;
- tests на создание новой БД;
- tests на backward migration policy если она будет поддерживаться;
- repository tests;
- tests на корректность materialized state;
- tests на transactional behavior в ключевых use cases.

## Требования к документации

Обязательна документация:

- схема таблиц;
- описание миграций;
- список repository contracts;
- правила работы с транзакциями;
- политика хранения сырых сообщений и событий.

## Критерии готовности

- БД инициализируется из библиотеки;
- миграции автоматизируемы;
- materialized views поддерживаются явно;
- repository API стабилен;
- тесты покрывают хранение и чтение критичных данных.

Для текущего этапа второго прохода practically считаются уже закрытыми:

- поддержка richer sync journal для `raw_events`;
- поддержка очереди `deferred_events` для reprocessing;
- безопасная file-based персистентность под временным `HOME`.

Но модуль еще не считается завершенным:

- нет отдельной migration form beyond current schema bootstrap;
- индексы и более явные sync-oriented read APIs пока минимальны;
- apply/sync statuses еще не оформлены как более строгая typed persistence model.

## Связанные документы

- [idea.technical.md](../../docs/idea.technical.md)
- [tauri-client-structure.md](../../docs/tauri-client-structure.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
