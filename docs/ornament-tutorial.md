# Tutorial: как использовать `Ornament` в LiveLetters

## Зачем нужен этот документ

Документ фиксирует не абстрактный обзор `Ornament`, а именно тот способ, которым он уже встроен в текущий frontend LiveLetters.

## Что принято в текущем проекте

В `apps/liveletters-frontend-app` `Ornament` используется как compile-time источник CSS.

Это означает:

- стили описываются через `defstyled`;
- CSS не генерируется в browser runtime;
- итоговый CSS пишется в статический файл;
- этот файл подключается в HTML как обычный stylesheet.

Текущее подключение сделано в:

- [deps.edn](../apps/liveletters-frontend-app/deps.edn)
- [shadow-cljs.edn](../apps/liveletters-frontend-app/shadow-cljs.edn)
- [styles.clj](../apps/liveletters-frontend-app/src/liveletters/frontend_app/styles.clj)
- [theme.cljc](../apps/liveletters-frontend-app/src/liveletters/frontend_app/theme.cljc)
- [resources/index.html](../apps/liveletters-frontend-app/resources/index.html)

## Где что лежит

### 1. Глобальный слой

Файл:

- [styles.clj](../apps/liveletters-frontend-app/src/liveletters/frontend_app/styles.clj)

Там живет:

- строковый `global-styles`;
- функция `write-styles!`;
- `shadow-cljs` build hook `write-styles-hook`;
- CLI entry point `-main` для ручной генерации CSS.

Этот слой нужен для:

- reset/base styles;
- фонового оформления;
- общих `ll-*` классов;
- media rules, которые пока проще держать как plain CSS.

### 2. Ornament theme-компоненты

Файл:

- [theme.cljc](../apps/liveletters-frontend-app/src/liveletters/frontend_app/theme.cljc)

Там должны жить:

- layout wrappers;
- screen-level composition helpers;
- ornament components, которые не тянут runtime-specific namespaces.

Важно:

- этот namespace должен оставаться чистым от зависимостей на `store`, `runtime`, `js/*` и прочий browser-specific код;
- причина в том, что [styles.clj](../apps/liveletters-frontend-app/src/liveletters/frontend_app/styles.clj) загружает его на Clojure-стороне для генерации CSS.

Если ornament-компонент потянет namespace, который компилируется только как CLJS, ручная команда генерации CSS начнет падать.

## Как добавить новый ornament-компонент

### Шаг 1. Помести его в `theme.cljc`

Пример:

```clojure
(ns liveletters.frontend-app.theme
  (:require [lambdaisland.ornament :as o]))

(o/defstyled hero-note :div
  {:padding "16px"
   :border-radius "16px"
   :background "rgba(255,255,255,0.72)"})
```

### Шаг 2. Используй его в обычном CLJC/CLJS UI-коде

Пример:

```clojure
(ns liveletters.frontend-app.pages
  (:require [liveletters.frontend-app.theme :as theme]))

[theme/hero-note {}
 "Hello"]
```

### Шаг 3. Пересобери CSS

Локально в `apps/liveletters-frontend-app`:

```bash
clojure -M:css
```

После этого обновится:

- [ornament.css](../apps/liveletters-frontend-app/resources/ornament.css)

## Как здесь работает build pipeline

### Ручная генерация

Команда:

```bash
clojure -M:css
```

Что делает:

- загружает `theme.cljc`;
- собирает `Ornament` registry через `o/defined-styles`;
- пишет итог в `resources/ornament.css`.

### Browser build

Команда:

```bash
clojure -M:app
```

Что делает:

- компилирует frontend через `shadow-cljs`;
- на стадии `:flush` вызывает `write-styles-hook`;
- снова пишет `resources/ornament.css`.

То есть даже если забыли вручную вызвать `:css`, нормальный browser build все равно перепишет generated stylesheet.

## Что можно делать через `Ornament`, а что лучше не делать

### Хорошие кандидаты

- app shell;
- navigation shells;
- page-level layout;
- grid wrappers;
- card wrappers;
- section-specific decorative containers;
- переиспользуемые куски screen composition.

### Плохие кандидаты в текущей архитектуре

- код, который зависит от `store.cljc`, если там есть `js/...`;
- код, который тянет runtime adapters;
- все, что невозможно загрузить на Clojure-стороне без CLJS-only окружения.

## Почему часть CSS остается plain CSS

В этой интеграции не всё перенесено в `defstyled`.

Часть базового слоя пока остается в `global-styles`, потому что так проще и безопаснее держать:

- глобальные reset/base rules;
- shared `ll-*` классы из существующего UI;
- media rules;
- bridge-слой между старым CSS contract и новым ornament contour.

Это осознанное переходное решение, а не ошибка.

## Как не сломать генерацию CSS

Проверочный список перед завершением работы:

1. `theme.cljc` не тянет runtime-specific namespaces.
2. `clojure -M:css` проходит.
3. `clojure -M:app` проходит.
4. `clojure -M:test` проходит.
5. В [ornament.css](../apps/liveletters-frontend-app/resources/ornament.css) появился ожидаемый новый class block.

## Что делать, если хочется больше ornament и меньше plain CSS

Безопасный порядок такой:

1. сначала переносить screen/layout wrappers;
2. затем переносить page-local cards и grids;
3. только потом решать, нужно ли переносить базовые `ll-button`, `ll-input`, `ll-field`;
4. не смешивать в одной правке одновременно:
   - большой visual redesign;
   - перенос build pipeline;
   - перенос большого числа primitive components.

## Связанные документы

- [build.md](../docs/build.md)
- [0002-development-invariants-and-internal-documents.md](../docs/adr/0002-development-invariants-and-internal-documents.md)
- [0005-incremental-reports-in-docs.md](../docs/adr/0005-incremental-reports-in-docs.md)
