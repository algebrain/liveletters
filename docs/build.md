# Сборка LiveLetters

## Назначение

Этот документ фиксирует минимально рабочий порядок сборки и проверки LiveLetters в текущем состоянии репозитория.

Он описывает именно фактически проверенный путь:

- собрать frontend app;
- сгенерировать CSS для frontend;
- прогнать frontend tests;
- запустить desktop runtime через Tauri backend app.

## Текущая структура сборки

В репозитории нет общего workspace в корне.

Поэтому сборка и проверки запускаются на уровне конкретных подпроектов.

Для пользовательского desktop-приложения сейчас важны два подпроекта:

- `apps/liveletters-frontend-app`
- `apps/liveletters-rust-backend-app`

## Предварительные требования

Нужно иметь в среде:

- JVM и `clojure`;
- Node.js и пакетный менеджер, совместимый с frontend app;
- Rust toolchain и `cargo`;
- системные зависимости для Tauri/Linux, если запуск идет на Linux.

Для Linux prerequisites см.:

- [tauri-linux-prerequisites.md](./tauri-linux-prerequisites.md)

## Frontend app

Каталог:

- `apps/liveletters-frontend-app`

### 1. Сгенерировать CSS

```bash
cd apps/liveletters-frontend-app
clojure -M:css
```

Что делает команда:

- загружает ornament theme;
- собирает compile-time CSS;
- пишет результат в `resources/ornament.css`.

### 2. Собрать browser bundle

```bash
cd apps/liveletters-frontend-app
clojure -M:app
```

Что делает команда:

- запускает `shadow-cljs` browser build;
- пишет JS bundle в `resources/js`;
- на build hook дополнительно обновляет `resources/ornament.css`.

### 3. Прогнать frontend tests

```bash
cd apps/liveletters-frontend-app
clojure -M:test
```

## Rust backend app

Каталог:

- `apps/liveletters-rust-backend-app`

### 1. Прогнать backend tests

```bash
cd apps/liveletters-rust-backend-app
cargo test -q
```

### 2. Запустить desktop runtime

```bash
cd apps/liveletters-rust-backend-app
./run-tauri-runtime.sh
```

Что делает script:

- архивирует предыдущие runtime logs;
- выставляет `LIVELETTERS_RUNTIME_LOG_DIR`;
- запускает `cargo run --features tauri-runtime`.

Логи runtime складываются в:

- `.docs/runtime-logs/`

## Минимальный проверенный порядок

Если нужен честный локальный прогон пользовательского desktop-приложения, текущий практический порядок такой:

```bash
cd apps/liveletters-frontend-app
clojure -M:css
clojure -M:app
clojure -M:test

cd ../liveletters-rust-backend-app
cargo test -q
./run-tauri-runtime.sh
```

## Что важно помнить

- frontend app не использует runtime styling engine для `Ornament`;
- generated CSS должен существовать как обычный файл `resources/ornament.css`;
- `clojure -M:app` сам обновляет generated CSS через build hook, но ручной `clojure -M:css` полезен как отдельная явная проверка;
- если менялись только Rust-модули, frontend build не всегда нужен;
- если менялись frontend styles или ornament theme, `clojure -M:css` и `clojure -M:app` нужно прогонять обязательно.

## Связанные документы

- [workspace-structure.md](./workspace-structure.md)
- [tauri-client-structure.md](./tauri-client-structure.md)
- [tauri-linux-prerequisites.md](./tauri-linux-prerequisites.md)
