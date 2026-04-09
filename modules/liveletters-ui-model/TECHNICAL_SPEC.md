# liveletters-ui-model

## Назначение

`liveletters-ui-model` это библиотека клиентских моделей представления. Она преобразует backend DTO в формы, удобные для экранов frontend-приложения.

Эта библиотека рассматривается как view-model слой для frontend-приложения, построенного по принятому UI-пути:

- `Reagent`

## Зона ответственности

- view model construction;
- selectors;
- formatters;
- grouping and sorting rules для UI;
- mapping backend DTO в screen-specific models;
- helper functions для экранов.

## Что модуль не должен делать

- напрямую вызывать backend;
- содержать доменную модель Rust;
- содержать маршрутизацию;
- содержать низкоуровневые UI components;
- превращаться в набор `Reagent`-компонентов.

## Основные сценарии

- построение feed view model;
- построение resource page model;
- построение post thread model;
- форматирование дат, статусов и видимости;
- нормализация ошибок для отображения;
- client-side sorting и filtering для экранов.

## Входные зависимости

- DTO из `liveletters-frontend-api`;
- опционально переиспользуемые UI-типажи из `liveletters-ui-kit` без жесткой сцепки.

## Требования к API

- чистые функции;
- отсутствие побочных эффектов;
- предсказуемые screen-specific модели;
- стабильные контракты для frontend-app;
- пригодность для работы поверх state slices из корневого `app-state`.

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- `feed-view-model`;
- `post-thread-view-model`;
- `sync-status-view-model`;
- `incoming-failures-view-model`;
- `event-failures-view-model`.

Текущий `sync-status-view-model` уже знает richer diagnostics contour:

- `duplicates`;
- `replays`;
- `unauthorized`;
- `invalid`;
- `malformed`;
- `deferred`;
- `outbox`.

Текущий diagnostics-related contour разделен на два разных screen-friendly списка:

- incoming failures;
- event failures.

## Роль относительно `Reagent` и `app-state`

`liveletters-ui-model` должен работать как слой между:

- DTO и app-state slices;
- и `Reagent`-компонентами frontend app.

Это означает:

- здесь допустимо знать форму данных, удобную для `Reagent`-экранов;
- здесь уместны selectors, formatters, sorting и mapping rules;
- но здесь не должны жить сами UI-компоненты и runtime-specific view effects.

## Требования к структуре каталога

- `src/feed`;
- `src/resource`;
- `src/post`;
- `src/thread`;
- `src/format`;
- `src/errors`;
- `test`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на selectors;
- tests на mapping DTO -> view model;
- tests на форматирование дат и статусов;
- tests на sorting и filtering;
- tests на edge cases пустых и частично заполненных DTO.

## Требования к документации

Обязательна документация:

- список view models;
- mapping rules;
- правила форматирования;
- ограничения client-side transforms;
- примеры использования в экранах.

## Критерии готовности

- библиотека состоит из чистых функций;
- модели представления устойчивы;
- контракты documented;
- тесты покрывают ключевые selectors и mapping.

Для текущего этапа второго прохода practically уже покрыты:

- richer sync status formatting;
- diagnostics-related mapping для message failures и event failures.

Но модуль еще не считается завершенным:

- нет более богатых selectors поверх app-store slices;
- formatting policy пока минимальна;
- screen-specific модели для более полного diagnostic UI еще не выделены.

## Связанные документы

- [workspace-structure.md](../../docs/workspace-structure.md)
