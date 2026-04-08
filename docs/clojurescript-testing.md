# Тестирование ClojureScript-подпроектов

## Назначение

Этот документ фиксирует принятую в LiveLetters техническую схему тестирования ClojureScript-подпроектов.

Документ нужен затем, чтобы при создании новых подпроектов не приходилось заново выяснять:

- как именно должны запускаться тесты;
- почему используется именно такая связка инструментов;
- какие каталоги должны попасть в `deps.edn`;
- как должен быть настроен `shadow-cljs`.


## Принятая схема

Для ClojureScript-подпроектов в LiveLetters используется следующая базовая схема:

- в `deps.edn` явно фиксируется версия `org.clojure/clojure`;
- тестовый запуск идет через `shadow-cljs`;
- в `deps.edn` перечисляются как каталоги исходников, так и каталоги тестов;
- в `shadow-cljs.edn` тестовая сборка настраивается как `:node-test`;
- пространства имен тестов должны оканчиваться на `-test`.

Эта схема выбрана по образцу рабочего проекта `../lcmm/lcmf-http`, где она уже доказала свою пригодность в текущей среде.


## Почему используется именно эта схема

### 1. Она работает в текущей среде без локального `.m2`

Для проекта уже принято решение не создавать локальный `.m2` внутри репозитория и использовать глобальное хранилище зависимостей.

Практически это означает, что проект не должен полагаться на случайное поведение среды. Версия `org.clojure/clojure` в `deps.edn` должна быть указана явно и должна соответствовать тому, что реально доступно в глобальном хранилище.

В текущей рабочей схеме используется:

- `org.clojure/clojure {:mvn/version "1.11.1"}`

### 2. `shadow-cljs` надежно запускает ClojureScript-тесты

Для ClojureScript-подпроектов в monorepo тестовый запуск через `shadow-cljs` оказался более надежной и предсказуемой схемой, чем попытка собирать отдельный самостоятельный запускатель тестов без него.

### 2.1. В модели независимых подпроектов возможны предупреждения о внешних путях

Если frontend app живет в `apps/`, а зависимые ClojureScript-модули в `modules/`, `deps.edn` может предупреждать о `:paths` вне каталога проекта.

Сейчас это считается приемлемой платой за выбранную модель независимых подпроектов внутри одного репозитория. Это не запрещает запуск тестов, но должно восприниматься как ожидаемая шероховатость такой схемы.

### 3. Каталоги тестов должны быть видны и `deps`, и `shadow-cljs`

Важная практическая деталь: недостаточно просто положить тесты в подпроект и перечислить test-paths только в `shadow-cljs.edn`.

Чтобы тестовые пространства имен реально подхватывались, каталоги тестов нужно включать и в `deps.edn`, и в `shadow-cljs.edn`.

Иначе возможна ситуация, когда сборка проходит, но `shadow-cljs` не находит ни одного теста и сообщает:

- `Ran 0 tests containing 0 assertions.`


## Обязательные правила для новых ClojureScript-подпроектов

### 1. Явно указывать `org.clojure/clojure` в `deps.edn`

Нельзя полагаться на неявную версию из среды.

Нужно явно указывать:

```clojure
:deps {org.clojure/clojure {:mvn/version "1.11.1"}}
```

Если в будущем стандартная версия будет изменена для всего репозитория, это должно быть сделано осознанно и единообразно.

### 2. Добавлять в `deps.edn` и `src`, и `test`

Если появляется новый ClojureScript-подпроект, его каталоги исходников и тестов должны быть перечислены в `:paths`.

Пример:

```clojure
{:paths ["liveletters-some-module/src"
         "liveletters-some-module/test"]}
```

Для monorepo это правило действует на уровне общего `deps.edn`.

### 3. Добавлять в `shadow-cljs.edn` и `src`, и `test`

Те же каталоги должны быть перечислены в `:source-paths` тестовой сборки `shadow-cljs`.

### 4. Использовать тестовую сборку `:node-test`

Базовая рекомендуемая конфигурация:

```clojure
{:builds
 {:test
  {:target :node-test
   :output-to ".shadow-cljs/node-tests.js"
   :ns-regexp "-test$"
   :autorun true}}}
```

### 5. Писать тесты как ClojureScript-тесты

Для test namespaces нужно использовать `cljs.test`, а не `clojure.test`.

Типовой пример:

```clojure
(ns liveletters.some-module.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.some-module.core :as core]))
```

### 6. Имена test namespaces должны оканчиваться на `-test`

Именно по этому признаку их подбирает `:ns-regexp "-test$"`.


## Базовый пример `deps.edn`

```clojure
{:paths ["liveletters-ui-kit/src"
         "liveletters-ui-kit/test"
         "liveletters-ui-model/src"
         "liveletters-ui-model/test"]
 :deps {org.clojure/clojure {:mvn/version "1.11.1"}}
 :aliases
 {:test
  {:extra-deps {thheller/shadow-cljs {:mvn/version "2.28.20"}}
   :main-opts ["-m" "shadow.cljs.devtools.cli" "compile" "test"]}}}
```


## Базовый пример `shadow-cljs.edn`

```clojure
{:source-paths ["liveletters-ui-kit/src"
                "liveletters-ui-kit/test"
                "liveletters-ui-model/src"
                "liveletters-ui-model/test"]
 :builds
 {:test
  {:target :node-test
   :output-to ".shadow-cljs/node-tests.js"
   :ns-regexp "-test$"
   :autorun true}}}
```


## Признаки неправильной настройки

Если схема настроена неправильно, типичные симптомы такие:

- `clojure -M:test` завершается ошибкой разрешения `org.clojure/clojure`;
- `shadow-cljs` компилирует проект, но пишет `Ran 0 tests`;
- тестовые пространства имен не попадают в сборку;
- тесты написаны через `clojure.test` вместо `cljs.test`;
- каталоги `test` перечислены только в одном месте и не видны всему тестовому контуру.


## Практическое правило для новых подпроектов

При добавлении нового ClojureScript-подпроекта нужно сразу проверить:

1. Его `src` добавлен в общий `deps.edn`.
2. Его `test` добавлен в общий `deps.edn`.
3. Его `src` добавлен в `shadow-cljs.edn`.
4. Его `test` добавлен в `shadow-cljs.edn`.
5. Его тесты используют `cljs.test`.
6. Его test namespaces оканчиваются на `-test`.
7. `clojure -M:test` после этого реально исполняет хотя бы один тест.


## Связанные документы

- [0003-test-lint-format-and-shared-utils.md](./adr/0003-test-lint-format-and-shared-utils.md)
- [0004-no-local-m2.md](./adr/0004-no-local-m2.md)
