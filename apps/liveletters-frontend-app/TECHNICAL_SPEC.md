# liveletters-frontend-app

## Назначение

`liveletters-frontend-app` это пользовательское frontend-приложение LiveLetters на ClojureScript. Оно отвечает за экраны, маршрутизацию, пользовательские сценарии и визуальное взаимодействие с backend.

Этот модуль является одним из двух верхнеуровневых приложений monorepo. Он должен быть тонким и опираться на библиотеки `liveletters-frontend-api`, `liveletters-ui-model` и `liveletters-ui-kit`.

## Зона ответственности

- маршрутизация приложения;
- composition экранов;
- пользовательские сценарии создания и редактирования постов и комментариев;
- экраны ленты, блога, поста, треда, подписок, друзей, ресурсов и настроек;
- обработка пользовательских действий;
- orchestration UI-состояния;
- интеграция с backend commands и backend events;
- экран синхронизации и экран диагностики.

## Что модуль не должен делать

- реализовывать доменную модель;
- содержать правила валидации протокольных сообщений;
- напрямую работать с IMAP, SMTP или SQLite;
- дублировать доменную логику Rust backend;
- хранить низкоуровневые детали протокола вне интеграционного слоя.

## Основные подсистемы

- `app shell` и bootstrap;
- `app-store` как клиентский application layer;
- routing;
- pages;
- feature-level UI flows;
- локальное UI state там, где оно не обязано жить в общем store;
- integration with frontend API;
- subscriptions to backend events;
- error presentation and recoverability.

## Основные экраны

- главная лента;
- страница ресурса;
- страница поста;
- тред комментариев;
- список подписок;
- список друзей;
- список ресурсов пользователя;
- настройки аккаунта;
- настройки почтовых подключений;
- синхронизация;
- диагностика.

## Входные зависимости

- `liveletters-frontend-api`;
- `liveletters-ui-model`;
- `liveletters-ui-kit`.

## Публичные контракты

- старт приложения;
- регистрация маршрутов;
- binding экранов к view model;
- подписка на app-level backend events;
- пользовательские intents, превращаемые в вызовы frontend API.

## Текущее минимальное состояние реализации

Сейчас модуль уже включает:

- единый `app-state`;
- route switching между:
  - feed
  - post thread
  - sync
  - diagnostics
- store-level intents и refresh functions для:
  - feed
  - sync status
  - incoming failures
  - event failures
- diagnostics page, которая уже показывает:
  - incoming failures;
  - event failures.

Текущий frontend app уже согласован с richer backend contour второго прохода:

- `sync-status` хранится в expanded форме;
- diagnostics state разделен на `incoming-failures` и `event-failures`;
- UI по-прежнему остается thin относительно `frontend-api` и `ui-model`.

## Требования к структуре каталога

- `src/app` для bootstrap и app shell;
- `src/pages` для экранов;
- `src/features` для пользовательских сценариев;
- `src/routes` для маршрутизации;
- `src/state` для UI state;
- `src/subscriptions` для подписок на backend events;
- `test` для пользовательских и UI integration tests.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на маршрутизацию основных страниц;
- tests на ключевые пользовательские сценарии;
- tests на обработку backend errors в UI;
- tests на экран создания поста;
- tests на экран треда комментариев;
- tests на экран синхронизации;
- smoke tests на startup приложения.

## Требования к документации

Обязательна документация:

- overview модуля;
- схема экранов;
- описание точек входа;
- описание интеграции с `liveletters-frontend-api`;
- соглашения по структуре UI-кода;
- правила управления состоянием.

## Правила управления состоянием

Для `liveletters-frontend-app` принимается консервативная state-centric схема:

- базовый UI-путь это `Reagent`;
- на frontend есть единый `app-store` как центр прикладного состояния и orchestration;
- базовая техническая форма этого store по умолчанию это один корневой `app-state` в `atom`;
- дополнительные `atom` допустимы только при явно обоснованной ответственности;
- не все вообще состояние обязано жить в app-store;
- локальное эфемерное UI-состояние допустимо держать рядом с компонентом;
- `liveletters-frontend-api` не хранит UI state, а только вызывает backend и подписывается на события;
- `liveletters-ui-model` дает чистые selectors и view model;
- orchestration пользовательских intents, effects и обновлений прикладного состояния живет в app-store внутри frontend app.

`re-frame` не считается обязательным базовым слоем и не вводится по умолчанию.

Практический смысл этой схемы:

- frontend app должен выглядеть не как набор разрозненных page handlers, а как единый клиентский application layer;
- экранный слой должен быть максимально разгружен;
- app-store должен скрывать backend-вызовы, orchestration и значимую часть прикладного поведения от версточного слоя.

Следствие для следующих этапов:

- если модулю `liveletters-ui-kit` или самому `liveletters-frontend-app` нужен `Reagent`, его нужно использовать прямо;
- не требуется искусственно держать frontend runtime-нейтральным, если это уже противоречит принятому стеку проекта.

## Критерии готовности

- приложение поднимается локально в development-режиме;
- основные страницы доступны;
- команды backend вызываются через `liveletters-frontend-api`;
- UI не зависит напрямую от деталей протокола и базы данных;
- тесты проходят;
- документация описывает ключевые сценарии.

Для текущего этапа второго прохода practically уже покрыты:

- app-state orchestration для richer diagnostics contour;
- diagnostics page под новый backend/frontend boundary;
- init path, который загружает feed, sync status, incoming failures и event failures.

Но модуль еще не считается завершенным:

- нет runtime bridge к реальному Tauri слою;
- нет richer event subscription contour;
- diagnostics UI пока остается минимальным и техническим.

## Связанные документы

- [tauri-client-structure.md](../../docs/tauri-client-structure.md)
- [workspace-structure.md](../../docs/workspace-structure.md)
