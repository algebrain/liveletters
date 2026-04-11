# E2E-контур для визуальной проверки страниц LiveLetters

## Назначение

Этот документ описывает использование Playwright для:

- визуальной проверки страниц приложения;
- снятия скриншотов любых экранов;
- итеративной настройки дизайна и вёрстки.

Контур работает через **моки Tauri API** — реальный Rust-backend не требуется.

## Структура файлов

Каталог:

- [`apps/liveletters-rust-backend-app/e2e/`](../apps/liveletters-rust-backend-app/e2e/)

```
e2e/
├── package.json              # Зависимости и скрипты
├── playwright.config.ts       # Конфигурация Playwright
├── tsconfig.json              # TypeScript
├── eslint.config.js           # ESLint
├── .gitignore                 # Исключения
├── helpers/
│   ├── fixtures.ts            # Фикстура с mock Tauri API
│   ├── selectors.ts           # Селекторы для поиска элементов
│   └── tauri-mock.ts          # Моки Tauri-команд и событий
└── tests/
    ├── bootstrap.e2e.ts       # Smoke-тест: initial setup → save → feed
    └── *.e2e.ts               # Дополнительные тесты
```

## Быстрый старт

### Команды

```bash
cd apps/liveletters-rust-backend-app/e2e

# Запустить все тесты
pnpm test

# Запустить конкретный тест
pnpm exec playwright test --grep "Initial Setup"

# Запустить с подробным выводом (диагностика)
pnpm run test:diagnostic

# Запустить UI-режим (интерактивный)
pnpm run test:ui

# Линтинг
pnpm run lint

# Форматирование
pnpm run format
```

### Зависимости

Первый запуск:

```bash
cd apps/liveletters-rust-backend-app/e2e
pnpm install
pnpm exec playwright install webkit
```

## Как это работает

### Архитектура

```
┌─────────────────────────────────────────────┐
│  Playwright (WebKit)                        │
│                                             │
│  page.addInitScript(TAURI_MOCK_SCRIPT)      │
│    ↓                                        │
│  window.__TAURI_INTERNALS__ = mock          │
│    ↓                                        │
│  Frontend загружается как обычный SPA        │
│  Вызывает invoke() → получает мок-данные    │
└─────────────────────────────────────────────┘
```

1. Playwright запускает **WebKit** (тот же движок, что у Tauri на Linux)
2. `page.addInitScript()` внедряет **mock Tauri API** до загрузки бандла
3. Фронтенд загружается через встроенный HTTP-сервер (`serve`)
4. ClojureScript вызывает `invoke()` → mock возвращает данные
5. Рендерится реальный UI → можно делать скриншоты

### Что мокается

Mock реализует все Tauri-команды, которые вызывает фронтенд:

| Команда | Поведение mock |
|---------|---------------|
| `get_bootstrap_state` | Возвращает `setup_completed: false` по умолчанию |
| `save_settings` | Сохраняет настройки, ставит `setup_completed: true` |
| `get_settings` | Возвращает сохранённые настройки |
| `get_home_feed` | Возвращает пустой список постов |
| `get_sync_status` | Возвращает статус "idle" с нулями |
| `list_incoming_failures` | Возвращает пустой список |
| `list_event_failures` | Возвращает пустой список |
| `emit` / `listen` | Эмитит события `sync-status-changed`, `feed-updated` |

## Как снимать скриншоты

### Базовый пример

```typescript
import { test, expect } from "../helpers/fixtures.js";

test("скриншот initial setup", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 2 })).toBeVisible();
  await page.screenshot({ path: "screenshots/initial-setup.png", fullPage: true });
});
```

### Скриншот с заполненными полями

```typescript
test("скриншот заполненной формы", async ({ page }) => {
  await page.goto("/");

  // Заполняем форму
  await page.getByRole("textbox", { name: "Nickname" }).fill("alice");
  await page.getByRole("textbox", { name: "Email" }).fill("alice@example.com");
  await page.getByRole("textbox", { name: "SMTP host" }).fill("smtp.example.com");
  await page.getByRole("textbox", { name: "SMTP port" }).fill("587");
  await page.getByRole("textbox", { name: "SMTP username" }).fill("alice");
  await page.getByRole("textbox", { name: "SMTP password" }).fill("secret");
  await page.getByRole("textbox", { name: "IMAP host" }).fill("imap.example.com");
  await page.getByRole("textbox", { name: "IMAP port" }).fill("143");
  await page.getByRole("textbox", { name: "IMAP username" }).fill("alice");
  await page.getByRole("textbox", { name: "IMAP password" }).fill("secret");

  await page.screenshot({ path: "screenshots/filled-form.png", fullPage: true });
});
```

### Скриншот после перехода на другую страницу

```typescript
test("скриншот feed после сохранения", async ({ page }) => {
  await page.goto("/");
  // ... заполнить форму ...
  await page.getByRole("button", { name: /Save/i }).click();
  await expect(page.getByRole("heading", { level: 2, name: "Home feed" })).toBeVisible();
  await page.screenshot({ path: "screenshots/feed-page.png", fullPage: true });
});
```

## Навигация по страницам

Фронтенд использует hash-роуты. Можно переходить напрямую:

| Страница | URL | Заголовок |
|----------|-----|-----------|
| Initial Setup | `/` | "Initial setup" |
| Feed | `/#/feed` | "Home feed" |
| Settings | `/#/settings` | "Settings" |
| Sync | `/#/sync` | "Sync" |
| Diagnostics | `/#/diagnostics` | "Diagnostics" |
| Post thread | `/#/post/<id>` | "Post thread" |

### Пример: скриншот feed страницы напрямую

```typescript
test("скриншот feed страницы", async ({ page }) => {
  await page.goto("/#/feed");
  await expect(page.getByRole("heading", { level: 2, name: "Home feed" })).toBeVisible();
  await page.screenshot({ path: "screenshots/feed.png", fullPage: true });
});
```

## Как добавлять данные для новых страниц

Если для новой страницы нужны данные, которых нет в mock — расширьте `tauri-mock.ts`.

### Пример: добавить посты для feed

```typescript
// В helpers/tauri-mock.ts, внутри invoke():
case 'get_home_feed':
  return {
    posts: [
      {
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "alice",
        created_at: 1700000000,
        body: "Первый тестовый пост с длинным текстом для проверки вёрстки ленты",
        visibility: "public",
        hidden: false,
      },
      {
        post_id: "post-2",
        resource_id: "blog-1",
        author_id: "bob",
        created_at: 1700001000,
        body: "Второй пост, чтобы проверить отображение списка",
        visibility: "public",
        hidden: false,
      },
    ],
  };
```

**Правило:** добавляйте моки по мере необходимости, не пытайтесь предугадать все данные заранее.

## Как менять вёрстку

1. Сделайте скриншот текущего состояния
2. Измените CSS/компоненты в соответствующих модулях:
   - Компоненты: `modules/liveletters-ui-kit/src/liveletters/ui_kit/core.cljc`
   - Страницы: `apps/liveletters-frontend-app/src/liveletters/frontend_app/pages.cljc`
   - Тема/стили: `apps/liveletters-frontend-app/src/liveletters/frontend_app/theme.cljc`
3. Пересоберите frontend:
   ```bash
   cd apps/liveletters-frontend-app
   clojure -M:css && clojure -M:app
   ```
4. Запустите тест снова, сделайте новый скриншот
5. Сравните до/после

## Фронтенд-сборка

Для работы e2e-тестов нужны собранные ресурсы фронтенда:

```bash
cd apps/liveletters-frontend-app
clojure -M:css    # Генерирует resources/ornament.css
clojure -M:app    # Собирает JS-бандл в resources/js/
```

Результат: `apps/liveletters-frontend-app/resources/` — эти файлы раздаёт Playwright.

Если изменились стили, компоненты или тема — пересоберите перед прогоном тестов.

## Типичный рабочий цикл

1. **Хочу посмотреть страницу** → пишу тест со скриншотом
2. **Запускаю** `pnpm test` → получаю PNG
3. **Вижу что поправить** → меняю `.cljc`
4. **Пересобираю** frontend → `clojure -M:css && clojure -M:app`
5. **Запускаю снова** → сравниваю скриншоты
6. Повторяю пока не понравится

## Полезные возможности Playwright

### Скриншот конкретного элемента

```typescript
const card = page.locator(".ll-settings-card");
await card.screenshot({ path: "screenshots/card.png" });
```

### Скриншот всей страницы (с прокруткой)

```typescript
await page.screenshot({ path: "screenshots/full.png", fullPage: true });
```

### Пауза для отладки

```typescript
await page.pause();  // Откроется интерактивный режим
```

### Запуск одного теста

```bash
pnpm exec playwright test --grep "скриншот"
```

## Связанные документы

- [build.md](./build.md) — сборка frontend
- [tauri-client-structure.md](./tauri-client-structure.md) — структура Tauri-клиента
