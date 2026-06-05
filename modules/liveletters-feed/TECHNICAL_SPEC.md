# `liveletters-feed` — TECHNICAL_SPEC

## 1. Цель

Крейт реализует команду `lltt feed`. Команда закрывает базовый сценарий: «импортировал письмо → открыл ленту → увидел пост». Без команды `feed` импорт через `lltt inbox import` не имел бы визуальной обратной связи.

## 2. Архитектура и зависимости

```
apps/lltt
   └─ clap-разбор → liveletters_feed::Args
                       └─ run(ctx, args)
                            ├─ Store::open_for_home_dir(&ctx.home)
                            ├─ app_core::get_home_feed(&store, GetHomeFeedQuery)
                            ├─ config::load_identity(&ctx.home, &ctx.identity_name)
                            └─ print_feed(&feed, &display, args.limit)
```

Зависимости (`Cargo.toml`): `liveletters-app-core`, `liveletters-config`, `liveletters-output`, `liveletters-store`, `clap` (с `derive`), `thiserror`.

## 3. Структура модуля

| Файл | Назначение |
|---|---|
| `src/args.rs` | `Args { limit: Option<usize> }` через `clap::Args` |
| `src/error.rs` | `FeedError { Store, AppCore, Config }` через `thiserror` |
| `src/print.rs` | `print_feed(&HomeFeed, &str, Option<usize>)` — форматирование в stdout; внутренняя `truncate_body` (макс. 80 символов). |
| `src/time.rs` | `unix_to_ymdhms(u64) → (y, m, d, h, mi, s)`, `format_unix_iso8601_utc(u64) → String`. Поддерживает 1970–2100. |
| `src/run.rs` | `run(ctx, args)` + `run_inner`; оборачивает `FeedError` в `Box<dyn Error + Send + Sync>`. |
| `src/lib.rs` | Реэкспорт + `summary() = "показать ленту текущего пользователя liveletters"`. |

## 4. Формат вывода

```
лента пользователя: <display_name>
постов: <total> (показано: <shown>)

┌─ пост #<post_id> от <author_id> (скрыт)   ← «(скрыт)» только если hidden=true
│  visibility: <visibility | —>
│  <ISO 8601 UTC>
│  <body, по строкам, с обрезкой до 80 символов>
└─
```

Если `posts.is_empty()` — печатается `(пусто)` вместо блоков постов. Шапка с `display_name` печатается всегда, даже для пустой ленты (это «лента пользователя X»).

## 5. Фильтрация по identity

Сейчас `get_home_feed` возвращает **все** посты из БД, без фильтрации по `author_id` или `resource_id`. Сознательное упрощение: пока других путей импорта нет, в БД лежат только посты, импортированные текущим пользователем. В будущем добавится опциональная фильтрация по `meta.subscriptions`, но это потребует переделки `get_home_feed` (сейчас он не принимает список подписок).

## 6. Unix-время без зависимостей

`time.rs` реализует `unix_to_ymdhms` вручную (без `chrono`/`time`), чтобы не тащить ~250 КБ транзитивных зависимостей ради одного места в коде. Поддерживает годы 1970–2100. Високосные годы по правилу: «год делится на 4, кроме кратных 100, но не 400». Точность до секунд; в будущем, если потребуется работа с датами в `liveletters-diagnostics`, переедем на `chrono`.

## 7. Сценарии ошибок

| Сценарий | Возврат |
|---|---|
| Нет `init` | `StoreError::StoreNotInitialized` (через `Store::open_for_home_dir`) → `FeedError::Store` |
| Нет identity-файла | `print_feed` печатает `ctx.identity_name` вместо `display_name` (не ошибка) |
| `display_name` пустое | то же — `ctx.identity_name` |
| `--limit 0` | `shown = 0`, шапка печатается, постов нет |
| `--limit > posts.len()` | `shown = posts.len()`, печатаются все |

## 8. Совместимость

- В ранних версиях команда была заглушкой, `Args` пустой, `FeedError` пустой enum, `run` возвращал `NotYetImplemented`.
- Сейчас: реальная реализация, см. выше.
- Запланировано: добавление `--since`/`--until` и опциональной фильтрации по подпискам.

## 9. Что НЕ делает

- Не показывает комментарии к постам (это `lltt thread`).
- Не показывает скрытые посты отдельно (скрытые видны, но с маркером `(скрыт)`).
- Не редактирует БД (только чтение).
- Не обращается к сети.
- Не показывает постов других идентичностей, даже если они есть в БД (`get_home_feed` берёт всё подряд).
