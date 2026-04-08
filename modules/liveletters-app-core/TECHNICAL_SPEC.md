# liveletters-app-core

## Назначение

`liveletters-app-core` это прикладная библиотека orchestration уровня use cases. Она связывает domain, protocol, store, sync и mail в конкретные сценарии приложения.

## Зона ответственности

- commands;
- queries;
- orchestration use cases;
- application services;
- application-level errors;
- coordination outbox and inbox flows;
- формирование backend read models;
- транзакционные прикладные сценарии.

## Что модуль не должен делать

- содержать Tauri-specific код;
- напрямую реализовывать IMAP/SMTP transport;
- быть местом хранения низкоуровневых SQL запросов;
- дублировать доменные инварианты из `liveletters-domain`.

## Основные use cases

- создание поста;
- редактирование поста;
- создание комментария;
- подписка на ресурс;
- импорт входящих протокольных сообщений;
- запуск синхронизации;
- чтение ленты;
- чтение страницы ресурса;
- чтение треда;
- чтение статуса синхронизации.

## Входные зависимости

- `liveletters-domain`;
- `liveletters-protocol`;
- `liveletters-store`;
- `liveletters-mail`;
- `liveletters-sync`.

## Требования к API

- commands и queries должны быть разделены явно;
- для внешнего слоя должны существовать стабильные DTO или result types;
- orchestration не должен протекать в Tauri-app;
- application errors должны быть пригодны для маппинга в UI.

## Требования к структуре каталога

- `src/commands`;
- `src/queries`;
- `src/services`;
- `src/errors`;
- `src/read_models`;
- `tests`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на ключевые use cases;
- tests на command/query boundaries;
- tests на error mapping;
- tests на orchestration outbox при создании поста;
- tests на чтение materialized state;
- integration tests с in-memory или test SQLite.

## Требования к документации

Обязательна документация:

- перечень use cases;
- список публичных commands и queries;
- структура application errors;
- диаграмма зависимостей;
- правила расширения use case слоя.

## Критерии готовности

- use cases живут в этой библиотеке, а не в app;
- orchestration покрыт тестами;
- command и query API согласованы;
- библиотека не зависит от Tauri;
- документация описывает пользовательские сценарии и backend contracts.

## Связанные документы

- [tauri-client-structure.md](../../docs/tauri-client-structure.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
