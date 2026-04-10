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

- включает временный debug logging через `LIVELETTERS_DEBUG_LOGS=1`;
- запускает `cargo run --features tauri-runtime -- ...`;
- при необходимости принимает:
  - `--home-dir=/some/path`

Если debug logging включен, runtime logs складываются в:

- `<effective-home>/.liveletters/runtime-logs/`

где `effective-home` это:

- обычный `HOME` пользователя;
- или каталог из `--home-dir=...`, если он передан.

Важно:

- runtime logs считаются временным диагностическим контуром;
- это не обязательная часть обычного будущего release-запуска;
- без debug logging runtime log dir не создается.

### 3. Режимы запуска runtime

#### Debug launcher

Для расследования дефектов и временных runtime logs:

```bash
cd apps/liveletters-rust-backend-app
./run-tauri-runtime.sh
```

Этот script:

- включает `LIVELETTERS_DEBUG_LOGS=1`;
- пишет временные debug logs в `<effective-home>/.liveletters/runtime-logs/`;
- принимает `--home-dir=...`.

#### Release-like запуск без runtime logs

Для запуска без временного диагностического контура:

```bash
cd apps/liveletters-rust-backend-app
cargo run --features tauri-runtime --
```

Если нужен отдельный home-каталог и при этом не нужны runtime logs:

```bash
cd apps/liveletters-rust-backend-app
cargo run --features tauri-runtime -- --home-dir=/tmp/liveletters-a
```

В таком режиме:

- `LIVELETTERS_DEBUG_LOGS` не включается;
- `runtime-logs/` не создается;
- приложение использует только обычный data-home и SQLite-базу.

#### Локальный запуск двух экземпляров

Для локальной отладки двух независимых экземпляров можно использовать локальный script:

```bash
cd apps/liveletters-rust-backend-app
./two.local.sh
```

Он использует уже подготовленные каталоги:

- `data.local/home-a`
- `data.local/home-b`

и запускает два экземпляра с:

- `--home-dir=<repo>/data.local/home-a`
- `--home-dir=<repo>/data.local/home-b`

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

Если нужен отдельный home-каталог для этого запуска:

```bash
cd apps/liveletters-rust-backend-app
./run-tauri-runtime.sh --home-dir=/tmp/liveletters-a
```

Если нужен release-like запуск без debug logs:

```bash
cd apps/liveletters-rust-backend-app
cargo run --features tauri-runtime -- --home-dir=/tmp/liveletters-a
```

## Что важно помнить

- frontend app не использует runtime styling engine для `Ornament`;
- generated CSS должен существовать как обычный файл `resources/ornament.css`;
- `clojure -M:app` сам обновляет generated CSS через build hook, но ручной `clojure -M:css` полезен как отдельная явная проверка;
- если менялись только Rust-модули, frontend build не всегда нужен;
- если менялись frontend styles или ornament theme, `clojure -M:css` и `clojure -M:app` нужно прогонять обязательно.
- `--home-dir=...` ведет себя как подмена `HOME` для приложения;
- `run-tauri-runtime.sh` это debug launcher, а не обещание постоянного runtime logging в будущих release-сценариях.
- release-like запуск должен идти без `LIVELETTERS_DEBUG_LOGS`, чтобы не создавать временные runtime logs без необходимости.

## Связанные документы

- [workspace-structure.md](./workspace-structure.md)
- [tauri-client-structure.md](./tauri-client-structure.md)
- [tauri-linux-prerequisites.md](./tauri-linux-prerequisites.md)
