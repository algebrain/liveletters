# liveletters-rust-backend-app

## Назначение

`liveletters-rust-backend-app` это backend и Tauri-host приложение LiveLetters на Rust. Оно собирает систему из библиотек monorepo и предоставляет frontend безопасную backend-границу.

## Зона ответственности

- запуск Tauri-приложения;
- инициализация конфигурации;
- wiring Rust-библиотек;
- регистрация Tauri commands;
- публикация backend events для frontend;
- запуск фоновых jobs;
- жизненный цикл приложения;
- graceful shutdown;
- интеграция с frontend bundle.

## Что модуль не должен делать

- содержать основную доменную логику;
- содержать SQL-логику вне thin integration points;
- смешивать transport, sync и domain в одном месте;
- быть местом, где "временно" живет бизнес-логика.

## Входные зависимости

- `liveletters-domain`;
- `liveletters-protocol`;
- `liveletters-store`;
- `liveletters-app-core`;
- `liveletters-mail`;
- `liveletters-sync`;
- `liveletters-diagnostics`.

## Основные подсистемы

- Tauri bootstrap;
- dependency container;
- command registration;
- event bridge;
- background job startup;
- configuration loading;
- error boundary.

## Основные команды

- `create_post`;
- `edit_post`;
- `create_comment`;
- `subscribe_to_resource`;
- `run_sync_now`;
- `get_home_feed`;
- `get_post_thread`;
- `get_sync_status`;
- `list_incoming_failures`.

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- `BackendApp` как thin integration container;
- явный wiring поверх:
  - `liveletters-store`
  - `liveletters-app-core`
  - `liveletters-diagnostics`
- backend boundary для:
  - `create_post`
  - `get_home_feed`
  - `get_post_thread`
  - `get_sync_status`
  - `list_incoming_failures`
  - `list_event_failures`

Текущий `get_sync_status` уже отражает richer diagnostics contour:

- `duplicate`;
- `replay`;
- `unauthorized`;
- `invalid`;
- `malformed`;
- `deferred`;
- `pending_outbox`.

Текущий backend app все еще остается runtime-neutral integration layer, а не настоящим Tauri bootstrap.

## Требования к структуре каталога

- `src/main.rs` или `src/lib.rs` как точка входа;
- `src/bootstrap` для wiring;
- `src/commands` для Tauri command boundary;
- `src/events` для app events;
- `src/jobs` для запуска background jobs;
- `tests` для integration и smoke tests.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- smoke test на успешную инициализацию приложения;
- integration tests на registration commands;
- tests на корректный wiring зависимостей;
- tests на старт фоновых задач;
- tests на error propagation в UI-friendly формат.

## Требования к документации

Обязательна документация:

- как приложение собирается;
- какие библиотеки подключает;
- как запускается в development-режиме;
- какие точки входа предоставляет frontend;
- как устроен lifecycle;
- как добавлять новые commands без протечки бизнес-логики.

## Критерии готовности

- приложение собирается как Tauri-host;
- backend commands доступны frontend;
- зависимости собираются через явный wiring;
- бизнес-логика остается в библиотеках;
- smoke и integration tests проходят;
- документация покрывает запуск и расширение модуля.

Для текущего этапа второго прохода practically уже покрыты:

- thin backend wiring поверх app-core и diagnostics;
- richer diagnostic/status boundary;
- event failure boundary для следующего frontend integration слоя.

Но модуль еще не считается завершенным:

- нет Tauri-specific command layer;
- нет runtime bridge и backend events для frontend;
- нет отдельного bootstrap/container split beyond текущий минимальный integration слой.

## Связанные документы

- [tauri-client-structure.md](../../docs/tauri-client-structure.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
- [tauri-linux-prerequisites.md](../../docs/tauri-linux-prerequisites.md)
