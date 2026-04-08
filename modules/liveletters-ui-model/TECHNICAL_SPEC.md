# liveletters-ui-model

## Назначение

`liveletters-ui-model` это библиотека клиентских моделей представления. Она преобразует backend DTO в формы, удобные для экранов frontend-приложения.

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
- содержать низкоуровневые UI components.

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
- стабильные контракты для frontend-app.

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

## Связанные документы

- `/home/algebrain/src/my/liveletters/docs/workspace-structure.md`
