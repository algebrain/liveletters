# `liveletters-app-core` INTERFACE

## Назначение

`liveletters-app-core` это внешний прикладной интерфейс backend-логики LiveLetters.

Если смотреть на уже пройденные модули по порядку:

- `liveletters-domain` задает предметную форму сущностей и событий;
- `liveletters-protocol` задает форму технического сообщения;
- `liveletters-store` дает persistence boundary;
- `liveletters-app-core` поверх этого собирает законченные прикладные сценарии.

Это модуль не про низкий уровень и не про UI. Его роль в другом:

- принять прикладную команду;
- выполнить нужный use case;
- при необходимости задействовать доменную модель, store, protocol и sync;
- вернуть понятный результат или прикладную ошибку.

Именно здесь появляется язык сценариев уровня приложения:

- создать пост;
- создать комментарий;
- скрыть пост;
- отредактировать комментарий;
- получить домашнюю ленту;
- получить тред поста;
- получить очередь outbox;
- переобработать deferred events.

## Где находится интерфейс

- crate: `liveletters-app-core`
- точка подключения: `src/lib.rs`

Наружу модуль экспортирует:

- `AppCore` как главный фасад use case слоя;
- command-типы;
- query-типы;
- result-типы;
- read models;
- `AppCoreError`.

## Что считается внешним интерфейсом этого модуля

С практической точки зрения внешний интерфейс `liveletters-app-core` это:

1. `AppCore::new(store)`;
2. набор методов `AppCore`, соответствующих use cases;
3. структуры command/query для входных данных;
4. result-типы команд;
5. read models для query-результатов;
6. `AppCoreError` как единый прикладной тип ошибки.

Именно этим интерфейсом должны пользоваться:

- backend app;
- интеграционные тесты прикладного слоя;
- любой верхний Rust-код, который хочет работать с системой на уровне сценариев, а не на уровне отдельных store-операций.

## Главный объект: `AppCore`

### Зачем нужен `AppCore`

`AppCore` это фасад прикладного слоя.

Он нужен затем, чтобы верхний код не собирал use case вручную из разрозненных деталей:

- не открывал сам доменные типы и store-операции в каждом месте;
- не дублировал orchestration;
- не повторял логику постановки сообщений в outbox;
- не смешивал command и query code прямо в runtime boundary.

Идея очень простая:

- если внешний слой хочет выполнить прикладной сценарий, он идет в `AppCore`;
- если внешний слой хочет читать готовое состояние для UI, он тоже идет в `AppCore`.

### Как создается `AppCore`

`AppCore::new(store: &Store) -> AppCore`

То есть `AppCore` не владеет хранилищем сам и не открывает его сам.

Это важная часть контракта:

- жизненный цикл `Store` контролируется внешним слоем;
- `AppCore` только использует уже готовый persistence boundary.

Такой интерфейс удобен:

- для backend app;
- для тестов;
- для интеграционных сценариев, где store поднимается отдельно.

## Почему здесь есть и commands, и queries

`liveletters-app-core` уже следует логике CQRS-lite:

- commands меняют состояние;
- queries читают уже подготовленное состояние.

Это не жесткий отдельный фреймворк, а просто важная граница ответственности.

Польза от этого интерфейса такая:

- внешний слой понимает, какие операции являются изменяющими;
- а какие только читают materialized state;
- read models можно держать стабильными и удобными для верхнего boundary;
- orchestration модифицирующих сценариев не смешивается с логикой чтения.

## Командный интерфейс: что делает каждая команда

Сейчас наружу экспортируются такие command-типы:

- `CreatePostCommand`
- `CreateCommentCommand`
- `HidePostCommand`
- `EditCommentCommand`
- `ReprocessDeferredEventsCommand`

## `CreatePostCommand`

### Зачем нужна эта команда

`CreatePostCommand` описывает входные данные use case создания поста.

Поля:

- `post_id`
- `resource_id`
- `author_id`
- `created_at`
- `body`

Смысл команды:

- внешний слой формулирует намерение “создать пост”;
- `AppCore` берет на себя всё остальное.

### Что реально делает use case `create_post`

Когда вызывается:

- `AppCore::create_post(CreatePostCommand)`

модуль:

1. создает доменные идентификаторы и `PostBody`;
2. строит доменную сущность `Post`;
3. сохраняет материализованный `PostRecord`;
4. строит доменное событие `PostCreated`;
5. строит `ProtocolMessage`;
6. кодирует его;
7. сохраняет `OutboxRecord`.

То есть это не просто “запиши пост в базу”, а полный прикладной сценарий создания исходящего события.

## `CreateCommentCommand`

### Зачем нужна эта команда

`CreateCommentCommand` описывает входные данные use case создания комментария.

Поля:

- `comment_id`
- `post_id`
- `parent_comment_id`
- `author_id`
- `created_at`
- `body`

### Что реально делает use case `create_comment`

Когда вызывается:

- `AppCore::create_comment(CreateCommentCommand)`

модуль:

1. проверяет, что пост существует;
2. строит доменные идентификаторы и `CommentBody`;
3. создает доменную сущность `Comment`;
4. сохраняет материализованный `CommentRecord`;
5. строит событие `CommentCreated`;
6. строит `ProtocolMessage`;
7. ставит сообщение в outbox.

То есть команда не ограничивается созданием локального комментария, а сразу выражает его как исходящее доменное изменение.

## `HidePostCommand`

### Зачем нужна эта команда

`HidePostCommand` выражает намерение скрыть существующий пост.

Поля:

- `post_id`
- `actor_id`
- `created_at`

### Что реально делает use case `hide_post`

Когда вызывается:

- `AppCore::hide_post(HidePostCommand)`

модуль:

1. читает текущий `PostRecord`;
2. проверяет, что пост существует;
3. строит доменную сущность поста;
4. применяет доменную операцию `hide`;
5. сохраняет обновленный `PostRecord`;
6. создает событие `PostHidden`;
7. собирает protocol message;
8. ставит его в outbox.

То есть это use case “изменить состояние поста и зафиксировать это как внешнее событие”.

## `EditCommentCommand`

### Зачем нужна эта команда

`EditCommentCommand` выражает намерение отредактировать существующий комментарий.

Поля:

- `comment_id`
- `actor_id`
- `created_at`
- `body`

### Что реально делает use case `edit_comment`

Когда вызывается:

- `AppCore::edit_comment(EditCommentCommand)`

модуль:

1. читает текущий `CommentRecord`;
2. проверяет, что комментарий существует;
3. строит доменную сущность комментария;
4. применяет доменную операцию `edit`;
5. сохраняет новый `CommentRecord`;
6. строит событие `CommentEdited`;
7. кодирует protocol message;
8. ставит его в outbox.

То есть это прикладной сценарий редактирования комментария с обязательной постановкой соответствующего сообщения в исходящий контур.

## `ReprocessDeferredEventsCommand`

### Зачем нужна эта команда

Это специальная прикладная команда для повторной обработки отложенных событий.

У нее нет полей, потому что она выражает не операцию над одним объектом, а системное намерение:

- попробовать заново применить то, что раньше пришлось отложить.

### Что реально делает use case `reprocess_deferred_events`

Когда вызывается:

- `AppCore::reprocess_deferred_events(ReprocessDeferredEventsCommand)`

модуль:

1. поднимает `SyncEngine` поверх того же `Store`;
2. вызывает повторную обработку deferred queue;
3. собирает агрегированную прикладную сводку результата;
4. возвращает ее наружу.

То есть `app-core` здесь играет роль orchestration-слоя поверх `sync`, а не просто проксирует store.

## Result-типы команд: зачем они нужны

Сейчас наружу экспортируются:

- `CreatePostResult`
- `CreateCommentResult`
- `HidePostResult`
- `EditCommentResult`
- `ReprocessDeferredEventsResult`

Смысл этих типов в том, что команда возвращает не “какую-нибудь карту”, а структурированный результат use case.

## `CreatePostResult`

Содержит:

- созданный доменный `post`;
- созданное доменное событие `event`.

Это полезно, если верхнему коду нужно:

- проверить итоговое доменное состояние;
- использовать событие дальше;
- писать точные интеграционные тесты.

Чтение идет через:

- `post()`
- `event()`

## `CreateCommentResult`

Тот же смысл, но для комментария:

- `comment()`
- `event()`

## `HidePostResult`

Возвращает:

- новый скрытый `post`;
- событие `PostHidden`.

## `EditCommentResult`

Возвращает:

- новый `comment` после редактирования;
- событие `CommentEdited`.

## `ReprocessDeferredEventsResult`

Возвращает не доменную сущность, а summary:

- `summary()`

Это уже результат системного технического сценария, а не пользовательского изменения одной сущности.

## Query-интерфейс: что делает каждый запрос

Сейчас наружу экспортируются:

- `GetHomeFeedQuery`
- `GetPostThreadQuery`
- `GetPendingOutboxQuery`

## `GetHomeFeedQuery`

Это маркерный query-type без полей.

Он нужен, чтобы явно выразить намерение:

- прочитать домашнюю ленту.

Когда вызывается:

- `AppCore::get_home_feed(GetHomeFeedQuery)`

модуль:

- читает все посты из store;
- преобразует их в `PostSummary`;
- собирает `HomeFeed`.

Это read-only сценарий. Он не меняет состояние и не запускает дополнительную orchestration-логику.

## `GetPostThreadQuery`

Этот query-type содержит:

- `post_id`

Он нужен, чтобы выразить намерение:

- прочитать один пост и все его комментарии.

Когда вызывается:

- `AppCore::get_post_thread(GetPostThreadQuery { post_id })`

модуль:

- проверяет, что пост существует;
- читает сам пост;
- читает комментарии этого поста;
- собирает `PostThread`.

Это основной read-only use case для экрана поста.

## `GetPendingOutboxQuery`

Это query-type без полей для чтения текущего исходящего контура.

Когда вызывается:

- `AppCore::get_pending_outbox(GetPendingOutboxQuery)`

модуль:

- читает outbox entries из store;
- собирает `PendingOutbox`.

Этот запрос нужен в первую очередь для технического и диагностического контура, а не для пользовательского контента.

## Read models: зачем они нужны

Сейчас наружу экспортируются:

- `PostSummary`
- `CommentSummary`
- `OutboxEntry`
- `HomeFeed`
- `PostThread`
- `PendingOutbox`
- `DeferredReprocessingSummary`

Это read models прикладного слоя.

Их смысл:

- не отдавать наружу сырые store records;
- не отдавать наружу доменные сущности там, где нужен read-only projection;
- дать стабильную форму данных для backend boundary.

## `PostSummary`

Это компактное read-only представление поста.

Оно содержит:

- идентификаторы;
- время;
- body;
- visibility;
- hidden.

Используется:

- в `HomeFeed`;
- в `PostThread`;
- дальше маппится в backend DTO.

## `CommentSummary`

Это компактное read-only представление комментария.

Используется для thread query и последующего backend DTO layer.

## `OutboxEntry`

Это read model одной записи outbox.

Она нужна, когда верхний слой хочет не просто знать, что outbox существует, а читать его как список нормальных элементов.

## `HomeFeed`

Это контейнер для списка постов ленты.

Создается через:

- `HomeFeed::new(posts)`

Читается через:

- `posts()`

## `PostThread`

Это контейнер для одного поста и его комментариев.

Создается через:

- `PostThread::new(post, comments)`

Читается через:

- `post()`
- `comments()`

## `PendingOutbox`

Это контейнер для списка исходящих сообщений.

Создается через:

- `PendingOutbox::new(entries)`

Читается через:

- `entries()`

## `DeferredReprocessingSummary`

Это прикладная сводка результата повторной обработки deferred events.

Она содержит открытые счетчики:

- `applied`
- `replayed`
- `unauthorized`
- `invalid`
- `still_deferred`

Смысл этого типа:

- не возвращать верхнему слою низкоуровневый `SyncReport`;
- а дать уже агрегированный результат use case.

## `AppCoreError`: что означает эта ошибка

`AppCoreError` это единый внешний тип ошибок прикладного слоя.

Сейчас он содержит:

- `Domain(DomainError)`
- `Protocol(ProtocolError)`
- `Sync(SyncError)`
- `Store(StoreError)`
- `PostNotFound { post_id }`
- `CommentNotFound { comment_id }`

### Как его понимать

Это не просто “любая ошибка”.

Смысл `AppCoreError` в том, чтобы верхний слой видел, на каком уровне сломался use case:

- доменная валидация;
- протокольная сборка;
- sync orchestration;
- persistence;
- отсутствие нужной сущности.

Особенно важны варианты:

- `PostNotFound`
- `CommentNotFound`

Они означают не инфраструктурный сбой, а прикладной факт: нужного объекта просто нет в текущем состоянии.

Для backend boundary это очень полезно, потому что такие ошибки потом можно отдельно маппить в `not_found`.

## Как верхние слои должны использовать этот модуль

Правильный сценарий работы с `liveletters-app-core` выглядит так:

1. внешний слой открывает или получает `Store`;
2. создает `AppCore::new(&store)`;
3. формирует command или query;
4. вызывает соответствующий метод `AppCore`;
5. получает result/read model или `AppCoreError`;
6. уже потом переводит это в backend DTO или UI boundary.

То есть `app-core` должен быть первой точкой входа для прикладной логики, а не `Store` напрямую.

## Что этот модуль специально не делает

Чтобы не было ложных ожиданий, важно зафиксировать границы.

Этот модуль сейчас не делает следующего:

- не открывает store сам;
- не является Tauri command boundary;
- не отдает JSON-friendly DTO для frontend;
- не отправляет письма сам по сети;
- не строит diagnostics snapshot;
- не выполняет полноценно весь runtime lifecycle приложения.

Иными словами:

- это уже прикладной слой;
- но еще не backend app и не transport/runtime layer.

## Что считать стабильным контрактом здесь

Если смотреть глазами потребителя crate, то наиболее значимыми частями интерфейса являются:

1. набор методов `AppCore`;
2. формы command и query типов;
3. формы command result и read model типов;
4. правило, что модифицирующие use cases ставят protocol message в outbox;
5. `ReprocessDeferredEventsCommand` как прикладной вход в deferred reprocessing;
6. `AppCoreError` как единый прикладной error boundary.

Изменение этих элементов почти наверняка требует согласованных правок в:

- backend app;
- интеграционных тестах use case слоя;
- frontend boundary, если он опирается на конкретные read models через backend DTO.

## Связанные документы

- [TECHNICAL_SPEC.md](./TECHNICAL_SPEC.md)
