# liveletters-diagnostics

## Назначение

`liveletters-diagnostics` это библиотека диагностики и технической наблюдаемости LiveLetters. Она собирает данные о состоянии синхронизации, входящих и исходящих сообщениях и ошибках обработки.

## Зона ответственности

- технические статусы;
- журналы ошибок;
- журналы входящих сообщений;
- журналы исходящих сообщений;
- представление ingest failures;
- представление sync health;
- данные для диагностического экрана.

## Что модуль не должен делать

- быть основным местом хранения бизнес-логики;
- зависеть от UI компонентов;
- реализовывать transport;
- подменять собой store или sync.

## Основные подсистемы

- log model;
- diagnostic snapshots;
- sync health reporting;
- message inspection helpers;
- integration DTO для backend diagnostics API.

## Входные зависимости

- `liveletters-store`;
- опционально `liveletters-sync` для sync reports.

## Требования к API

- чтение диагностических срезов;
- чтение ошибок ingest;
- чтение статуса outbox и inbox;
- чтение состояния фоновых jobs;
- безопасные представления ошибок для UI.

## Требования к структуре каталога

- `src/logs`;
- `src/reports`;
- `src/health`;
- `src/dto`;
- `tests`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на формирование sync health;
- tests на агрегацию ошибок;
- tests на sanitization чувствительных полей;
- tests на stable DTO для diagnostic UI.

## Требования к документации

Обязательна документация:

- какие данные считаются диагностическими;
- какие поля должны скрываться или маскироваться;
- как строятся health reports;
- как diagnostic API используется приложением.

## Критерии готовности

- библиотека умеет отдавать диагностические данные без протечки лишних деталей;
- чувствительные поля маскируются;
- формат diagnostic data стабилен;
- тесты покрывают health и error aggregation.

## Связанные документы

- `/home/algebrain/src/my/liveletters/docs/tauri-client-structure.md`
- `/home/algebrain/src/my/liveletters/docs/workspace-structure.md`
