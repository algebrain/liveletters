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

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- `DiagnosticsSnapshot` как стабильный диагностический срез;
- `SyncHealth` с richer counters для:
  - `applied`
  - `duplicate`
  - `replay`
  - `unauthorized`
  - `invalid`
  - `malformed`
  - `deferred`
  - `pending_outbox`
- diagnostics DTO для:
  - `raw_messages`
  - `raw_events`
  - `deferred_events`
  - `outbox_entries`
- sanitization preview-полей с маскировкой email-адресов.

Текущий health policy пока intentionally conservative:

- `Degraded` выставляется, если есть хотя бы один `malformed`, `unauthorized`, `invalid` или `deferred` case;
- `replay` и `duplicate` считаются важными техническими сигналами, но сами по себе не переводят health в `Degraded`.

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

Для текущего этапа второго прохода practically уже покрыты:

- richer sync health summary;
- raw event failures как отдельный diagnostics contour;
- stable DTO для backend diagnostic boundary.

Но модуль еще не считается завершенным:

- нет более тонкой health taxonomy beyond `Healthy` / `Degraded`;
- нет richer aggregation по outbox/inbox/deferred подсистемам;
- sanitization policy пока остается минимальной.

## Связанные документы

- [tauri-client-structure.md](../../docs/tauri-client-structure.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
