# `liveletters-frontend-api` INTERFACE

## Назначение

`liveletters-frontend-api` это внешний программный интерфейс frontend-интеграции с backend LiveLetters.

Если смотреть на уже пройденные модули по порядку:

- backend app предоставляет Tauri commands, backend events и DTO на Rust-стороне;
- `liveletters-frontend-api` дает frontend-коду удобную и централизованную ClojureScript-границу для работы с этим backend boundary.

Смысл этого модуля не в том, чтобы содержать экранную логику или хранить состояние приложения. Его задача уже:

- вызывать backend-команды единообразно;
- подписываться на backend events;
- нормализовать backend-ответы;
- нормализовать ошибки;
- скрывать от остального frontend низкоуровневые детали конкретного runtime adapter.

То есть это frontend-side transport/integration layer между UI-приложением и backend app.

## Где находится интерфейс

- основной namespace: `liveletters.frontend-api.core`
- runtime adapter для Tauri: `liveletters.frontend-api.tauri`

Именно эти namespace образуют внешний интерфейс модуля.

## Что считается внешним интерфейсом этого модуля

С практической точки зрения внешний интерфейс `liveletters-frontend-api` это:

1. adapter contract;
2. helper-функции для вызова backend commands;
3. helper-функции для подписки на backend events;
4. функции нормализации DTO;
5. функция нормализации ошибок;
6. готовый `tauri-adapter` для реального runtime.

Именно этим API должен пользоваться frontend app, а не прямыми вызовами `@tauri-apps/api/core` и `@tauri-apps/api/event`.

## Главная идея модуля

На уровне архитектуры этот модуль решает две проблемы.

### 1. Не пускать runtime-детали в UI-код

Если вызывать `invoke`, `listen` и `emit` прямо из экранов и store-слоя, frontend быстро теряет границы ответственности.

`liveletters-frontend-api` нужен затем, чтобы:

- UI и app-store не знали о низкоуровневом Tauri API;
- backend commands были собраны в одном месте;
- формат ошибок и DTO был централизован.

### 2. Сделать backend boundary мокируемым

Если frontend-код работает только через adapter contract, то его проще:

- тестировать;
- подменять fake-реализацией;
- запускать без реального runtime.

То есть этот модуль не только “обертка над invoke”, а еще и boundary для тестируемости.

## Adapter contract: ключевой интерфейс модуля

Самая важная часть интерфейса этого модуля это adapter map.

Все imperative helper-функции ожидают объект с такими ключами:

- `:invoke-command`
- `:subscribe-event`
- `:emit-event`

Ожидаемые сигнатуры:

- `(:invoke-command adapter) command payload on-success on-error`
- `(:subscribe-event adapter) event-name handler`
- `(:emit-event adapter) event-name payload`

## Зачем нужен именно такой adapter

Этот контракт нужен затем, чтобы frontend-слой работал не с конкретным runtime, а с абстракцией:

- “вызвать backend-команду”;
- “подписаться на backend-событие”;
- “эмитить событие”.

Для кода экрана или app-store не должно быть важно, реализовано это через:

- Tauri;
- тестовый fake adapter;
- другую runtime-среду в будущем.

То есть adapter contract это главный внешний stable API этого модуля.

## Namespace `liveletters.frontend-api.core`

Это основной описательный и runtime-neutral слой.

Он не должен знать о деталях реализации Tauri API больше, чем необходимо для работы через adapter contract.

Наружу этот namespace отдает:

- `module-info`
- DTO helpers
- error normalization
- command helpers
- event subscription helpers

## `module-info`

### Зачем нужна эта функция

`module-info` возвращает простую метаинформацию о модуле:

- `{:module :liveletters-frontend-api :language :cljc}`

Это не прикладной use case, а служебная идентификация модуля.

## DTO helpers: зачем они нужны

Наружу экспортируются:

- `create-post-request`
- `sync-status-dto`
- `event-failure-dto`

Их смысл в том, чтобы нормализовать boundary между frontend и backend.

### `create-post-request`

Эта функция собирает map запроса для backend-команды создания поста.

Она возвращает структуру с ключами:

- `:post-id`
- `:resource-id`
- `:author-id`
- `:created-at`
- `:body`

Зачем это нужно:

- не собирать payload вручную в каждом экране;
- иметь одно место, где зафиксирована форма запроса;
- дать frontend-коду выразительный helper уровня намерения “создать запрос на создание поста”.

### `sync-status-dto`

Эта функция принимает backend-ответ и приводит его к frontend-friendly форме.

Она возвращает map с ключами:

- `:status`
- `:applied-messages`
- `:duplicate-messages`
- `:replayed-messages`
- `:unauthorized-messages`
- `:invalid-messages`
- `:malformed-messages`
- `:deferred-events`
- `:pending-outbox`

Зачем это нужно:

- backend использует snake_case JSON-поля;
- frontend-коду удобнее работать с keyword-ключами и согласованным стилем именования;
- normalization не должна быть размазана по экранам.

### `event-failure-dto`

Эта функция нормализует один backend-элемент event-failure списка.

Она возвращает map с ключами:

- `:event-id`
- `:event-type`
- `:resource-id`
- `:apply-status`
- `:failure-reason`

Смысл здесь такой же:

- привести backend DTO к frontend-форме;
- не заставлять UI знать backend snake_case поля напрямую.

## `normalize-error`: что делает эта функция

`normalize-error` это центральная функция приведения backend errors к frontend-форме.

Сейчас она распознает как минимум такие backend codes:

- `"validation_error"`
- `"not_found"`

И возвращает более удобную для frontend структуру:

- `{:type :validation ...}`
- `{:type :not-found ...}`
- либо `{:type :unknown ...}`

### Зачем это нужно

Если frontend будет работать напрямую с сырым `CommandErrorDto`, то логика обработки ошибок быстро расползется по приложению.

`normalize-error` нужен затем, чтобы:

- дать frontend единый словарь типов ошибок;
- отделить transport/backend codes от внутренней frontend-логики;
- сделать обработку ошибок предсказуемой.

Важно понимать ограничение:

- сейчас error taxonomy еще минимальна;
- но именно эта функция является официальной точкой, где она должна развиваться.

## Базовые imperative helpers

Наружу экспортируются:

- `invoke-command!`
- `subscribe-event!`
- `emit-event!`

Это не готовые пользовательские сценарии, а универсальные primitive-операции поверх adapter contract.

### `invoke-command!`

Смысл этой функции:

- взять adapter;
- вызвать на нем backend command;
- передать payload;
- обработать успех и ошибку через callbacks.

Это самый низкий рабочий уровень frontend integration boundary.

### `subscribe-event!`

Смысл этой функции:

- централизованно подписаться на backend event через adapter.

Она нужна, чтобы даже подписки на события шли через единый frontend API слой.

### `emit-event!`

Смысл этой функции:

- централизованно эмитить событие через adapter.

На текущем этапе это менее важный путь, чем invoke и subscribe, но он входит в общий контракт adapter map.

## Готовые command/query helpers

Наружу экспортируются:

- `create-post!`
- `get-home-feed!`
- `get-post-thread!`
- `get-sync-status!`
- `list-incoming-failures!`
- `list-event-failures!`

Их смысл:

- frontend-код должен работать не со строковыми именами команд напрямую;
- а через именованные helper-функции, отражающие прикладной смысл операции.

## `create-post!`

### Что делает

Вызывает backend command:

- `"create_post"`

Используется, когда frontend хочет инициировать создание нового поста.

На вход получает:

- adapter;
- request;
- `on-success`;
- `on-error`.

Смысл helper-функции:

- зафиксировать, что именно эта операция соответствует backend command `create_post`;
- не повторять строковый литерал по коду приложения.

## `get-home-feed!`

### Что делает

Вызывает backend command:

- `"get_home_feed"`

И дополнительно нормализует успешный ответ так, чтобы в `on-success` приходил сразу список постов:

- `(or (:posts response) [])`

То есть helper скрывает от верхнего слоя контейнерную форму backend DTO там, где это удобно.

## `get-post-thread!`

### Что делает

Вызывает backend command:

- `"get_post_thread"`

и передает дальше thread response как есть, уже в frontend map-форме.

Используется для экрана поста и комментариев.

## `get-sync-status!`

### Что делает

Вызывает backend command:

- `"get_sync_status"`

И дополнительно пропускает ответ через:

- `sync-status-dto`

То есть внешний код сразу получает нормализованный status map, а не сырой backend DTO.

## `list-incoming-failures!`

### Что делает

Вызывает backend command:

- `"list_incoming_failures"`

и передает список дальше в `on-success`.

Используется в diagnostic UI контуре.

## `list-event-failures!`

### Что делает

Вызывает backend command:

- `"list_event_failures"`

И дополнительно маппит каждый элемент через:

- `event-failure-dto`

То есть верхний слой получает уже нормализованный список проблемных событий.

## Готовые event subscription helpers

Наружу экспортируются:

- `subscribe-sync-status-changed!`
- `subscribe-feed-updated!`

Их смысл:

- привязать frontend к именованным backend events через отдельные helper-функции;
- не разбрасывать строки `"sync-status-changed"` и `"feed-updated"` по приложению.

## `subscribe-sync-status-changed!`

Подписывает handler на backend event:

- `"sync-status-changed"`

Используется, когда frontend хочет:

- заново загрузить status panel;
- обновить diagnostics UI;
- инвалидировать соответствующий кусок app-store.

## `subscribe-feed-updated!`

Подписывает handler на backend event:

- `"feed-updated"`

Используется, когда frontend хочет:

- перечитать домашнюю ленту;
- обновить список постов в app-store.

## Namespace `liveletters.frontend-api.tauri`

Это конкретная runtime-реализация adapter contract для Tauri.

Если `core`-namespace задает абстрактный frontend integration API, то `tauri`-namespace дает рабочую реализацию поверх:

- `@tauri-apps/api/core`
- `@tauri-apps/api/event`

Наружу этот namespace экспортирует одну ключевую функцию:

- `tauri-adapter`

## `tauri-adapter`: зачем он нужен

`tauri-adapter` возвращает adapter map, совместимый с `liveletters.frontend-api.core`.

Это и есть готовый bridge между frontend-кодом и реальным Tauri runtime.

### Что он делает внутри

Он берет на себя несколько важных деталей:

- переводит frontend kebab-case ключи в snake_case payload для backend;
- вызывает `invoke` для backend-команд;
- декодирует JS-ответы обратно в ClojureScript map с keyword-ключами;
- прогоняет backend ошибки через `normalize-error`;
- подписывается на backend events через `listen`;
- умеет эмитить события через `emit`.

То есть `tauri-adapter` нужен затем, чтобы rest of frontend code вообще не думал об этой низкоуровневой механике.

## Почему есть отдельный adapter, а не прямые вызовы `invoke`

Потому что frontend boundary должен быть:

- централизованным;
- заменяемым;
- тестируемым.

Если `invoke` и `listen` используются напрямую из экранов или app-store, приложение быстро получает:

- жесткую связанность с runtime;
- трудные для тестирования side effects;
- хаотичную обработку ошибок и payload shape.

`liveletters-frontend-api` нужен именно затем, чтобы этого не происходило.

## Где проходит граница между `frontend-api`, `ui-model` и frontend app

Это одно из самых важных архитектурных различий.

### `frontend-api`

Отвечает за:

- вызов backend;
- подписку на backend events;
- normalization DTO и ошибок;
- adapter boundary.

### `ui-model`

Отвечает за:

- преобразование DTO в screen-friendly view model;
- selectors;
- presentation-level mapping.

### frontend app

Отвечает за:

- app-store;
- orchestration пользовательских сценариев;
- route-level и screen-level состояние;
- вызов `frontend-api` и применение `ui-model`.

То есть `frontend-api` не должен превращаться ни в UI-слой, ни в глобальный store.

## Как верхние слои должны использовать этот модуль

Правильный сценарий использования обычно такой:

1. frontend app создает `tauri-adapter`;
2. app-store или orchestration слой вызывает helper-функции из `liveletters.frontend-api.core`;
3. успешные ответы передаются дальше в `ui-model`;
4. ошибки проходят через `normalize-error` и затем уже обрабатываются приложением;
5. подписки на backend events тоже заводятся через этот модуль.

Для initial setup и settings flow это теперь означает:

6. сначала вызвать `get-bootstrap-state!`;
7. затем вызвать `get-settings!`;
8. затем сохранять форму через `save-settings!`.

То есть `liveletters-frontend-api` должен быть единственной точкой прямого общения frontend с backend boundary.

## Что этот модуль специально не делает

Чтобы не было ложных ожиданий, важно зафиксировать границы.

Этот модуль сейчас не делает следующего:

- не хранит состояние приложения;
- не строит screen-friendly view model;
- не рендерит UI;
- не маршрутизирует пользователя;
- не содержит бизнес-логику предметной модели;
- не заменяет frontend app-store.

Иными словами:

- это integration boundary;
- но не presentation layer и не application state layer.

## Что считать стабильным контрактом здесь

Если смотреть глазами потребителя модуля, то наиболее значимыми частями интерфейса являются:

1. adapter contract;
2. набор helper-функций для backend commands;
3. набор helper-функций для backend events;
4. формы normalized DTO;
5. `normalize-error` как единая точка frontend-side error mapping;
6. `tauri-adapter` как рабочая runtime-реализация.

Текущее дополнение к этому контракту:

7. `bootstrap-state-dto`;
8. `settings-dto`;
9. `save-settings-request`;
10. helper-функции initial setup и settings flow.

Изменение этих элементов почти наверняка требует согласованных правок в:

- frontend app;
- `liveletters-ui-model`;
- backend boundary, если меняются имена команд, событий или формы DTO.

## Связанные документы

- [TECHNICAL_SPEC.md](./TECHNICAL_SPEC.md)
