# liveletters-domain

## Назначение

`liveletters-domain` это основная библиотека предметной модели LiveLetters. Она определяет сущности, инварианты, value objects и типы доменных событий.

## Зона ответственности

- `account`, `resource`, `post`, `comment`, `subscription`, `membership`, `visibility`;
- устойчивые идентификаторы;
- доменные инварианты;
- доменные политики доступа и видимости;
- типы доменных событий;
- базовые операции над доменными сущностями.

## Что модуль не должен делать

- знать о Tauri;
- знать о IMAP, SMTP и MIME;
- знать о SQLite;
- содержать DTO для frontend;
- зависеть от UI или transport кода.

## Основные сущности

- `Account`;
- `Resource`;
- `Post`;
- `Comment`;
- `Subscription`;
- `Membership`;
- `RoleAssignment`;
- `VisibilityPolicy`.

## Основные value objects

- `AccountId`;
- `ResourceId`;
- `PostId`;
- `CommentId`;
- `EventId`;
- `DeliveryAddress`;
- `Timestamp`.

## Основные доменные события

- `PostCreated`;
- `PostEdited`;
- `PostHidden`;
- `CommentCreated`;
- `CommentEdited`;
- `SubscriptionChanged`;
- `FriendshipChanged`;
- `MembershipGranted`;
- `MembershipRevoked`;
- `UserBanned`.

## Требования к API

- публичный API должен быть маленьким и типизированным;
- инварианты должны проверяться на уровне конструктора или фабрик;
- недопустимые состояния не должны конструироваться легко;
- ошибки доменной валидации должны быть явными типами.

## Требования к структуре каталога

- `src/account`;
- `src/resource`;
- `src/post`;
- `src/comment`;
- `src/subscription`;
- `src/visibility`;
- `src/events`;
- `tests`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- unit tests на инварианты сущностей;
- tests на visibility rules;
- tests на создание и редактирование постов;
- tests на построение дерева комментариев;
- tests на недопустимые переходы состояния;
- tests на корректную генерацию доменных событий.

## Требования к документации

Обязательна документация:

- словарь предметных терминов;
- список сущностей и их инвариантов;
- описание доменных событий;
- гарантии и негарантии библиотеки;
- правила расширения модели.

## Критерии готовности

- библиотека не зависит от инфраструктуры;
- доменная модель типизирована и согласована;
- инварианты покрыты тестами;
- доменные события описаны и документированы;
- публичный API стабилен и мал.

## Связанные документы

- [idea.technical.md](../../docs/idea.technical.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
