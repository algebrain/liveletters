# liveletters-frontend-api

## Назначение

`liveletters-frontend-api` это интеграционная ClojureScript-библиотека, связывающая frontend с Rust backend через Tauri commands и backend events.

Эта библиотека предназначена для использования из frontend-приложения, построенного по принятому в проекте UI-пути:

- `Reagent`

## Зона ответственности

- вызов backend commands;
- подписка на backend events;
- frontend-side DTO boundary;
- преобразование низкоуровневых backend ошибок в UI-friendly формат;
- transport abstraction для frontend.

## Что модуль не должен делать

- содержать экранную логику;
- содержать доменную модель;
- содержать UI-компоненты;
- содержать routing и page orchestration;
- хранить глобальное UI-состояние приложения.

## Основные команды и запросы

- `create-post`;
- `edit-post`;
- `create-comment`;
- `subscribe-to-resource`;
- `run-sync-now`;
- `get-home-feed`;
- `get-resource-page`;
- `get-post-thread`;
- `get-sync-status`;
- `list-incoming-failures`.

## Основные события

- `feed-updated`;
- `thread-updated`;
- `sync-status-changed`;
- `ingest-failures-changed`.

## Требования к API

- единая точка вызова backend commands;
- типизированные или хотя бы формализованные DTO;
- единая схема ошибок;
- abstraction layer, которую можно замокать в frontend tests;
- минимальная зависимость от конкретного Tauri runtime;
- пригодность для использования из `Reagent`-приложения без протечки backend details в UI-компоненты.

## Технологическая граница

Хотя текущий frontend-путь в проекте это `Reagent`, сам `liveletters-frontend-api` не должен превращаться в библиотеку UI-компонентов.

Его роль:

- вызывать backend;
- подписываться на backend events;
- нормализовать DTO и ошибки;
- оставаться тонкой интеграционной границей для `Reagent`-frontend.

То есть `Reagent` здесь фиксируется как контекст использования, а не как повод переносить UI-runtime логику внутрь этого модуля.

## Требования к структуре каталога

- `src/commands`;
- `src/events`;
- `src/dto`;
- `src/errors`;
- `src/adapters`;
- `test`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на command adapters;
- tests на mapping backend errors;
- tests на event subscription layer;
- tests на mockable API boundary;
- tests на DTO shape guarantees.

## Требования к документации

Обязательна документация:

- список доступных commands;
- список backend events;
- схема DTO;
- формат ошибок;
- способ мокать библиотеку в tests.

## Критерии готовности

- frontend работает только через эту библиотеку;
- backend contracts централизованы;
- ошибки нормализуются единообразно;
- тесты покрывают adapters и subscriptions.

## Связанные документы

- [tauri-client-structure.md](../../docs/tauri-client-structure.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
