# liveletters-ui-kit

## Назначение

`liveletters-ui-kit` это библиотека базовых UI-компонентов и визуальной системы LiveLetters для ClojureScript frontend.

## Зона ответственности

- buttons;
- forms;
- inputs;
- list primitives;
- layout primitives;
- typography primitives;
- reusable UI blocks;
- визуальные соглашения и accessibility patterns.

## Что модуль не должен делать

- содержать маршрутизацию;
- содержать backend integration;
- содержать доменную бизнес-логику;
- зависеть от конкретных экранов приложения.

## Основные подсистемы

- tokens и theme variables;
- primitives;
- composed components;
- form helpers;
- empty, loading и error states;
- accessibility helpers.

## Требования к API

- переиспользуемые и композиционные компоненты;
- минимальный публичный surface;
- документированные props и состояния;
- единые соглашения по стилю и accessibility.

## Требования к структуре каталога

- `src/tokens`;
- `src/primitives`;
- `src/components`;
- `src/forms`;
- `src/a11y`;
- `test`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на rendering основных primitives;
- tests на accessibility critical states;
- tests на controlled/uncontrolled form elements если они будут;
- visual regression или snapshot tests там, где это оправдано.

## Требования к документации

Обязательна документация:

- список компонентов;
- правила композиции;
- соглашения по стилю;
- accessibility guidance;
- примеры использования.

## Критерии готовности

- компоненты переиспользуемы;
- визуальная система согласована;
- ключевые компоненты документированы;
- тесты покрывают базовое поведение и accessibility.

## Связанные документы

- [workspace-structure.md](../../docs/workspace-structure.md)
