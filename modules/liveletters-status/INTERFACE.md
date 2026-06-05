# `liveletters-status` INTERFACE

## Назначение

`liveletters-status` — библиотечный крейт, реализующий команду `lltt status`.
Команда печатает краткий отчёт о состоянии домашнего каталога: количество
постов, комментариев, отложенных событий, размер исходящей очереди и время
последней активности.

## Где находится интерфейс

- crate: `liveletters-status`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- `Args` — clap-аргументы команды (без полей, команда не принимает аргументов);
- `StatusError` — типизированные ошибки команды;
- `run(ctx, args) -> Result<(), Box<dyn Error + Send + Sync>>` — единая точка запуска;
- `print_status(&StatusCounts)` — функция печати сводки;
- `StatusCounts` — структура с пятью счётчиками;
- `CommandContext` (реэкспорт из `liveletters-output`);
- константы `COMMAND_NAME`, `summary()`, `crate_name()`.

## Сигнатура запуска

```rust
pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>>
```

## Формат вывода

Пять строк формата `key: value`:

```
постов: <u64>
комментариев: <u64>
отложенных: <u64>
исходящих: <u64>
последняя активность: <YYYY-MM-DD HH:MM:SS UTC> | нет активности
```

`последняя активность` берётся как `MAX(created_at)` из таблицы `posts`;
если постов нет — выводится `нет активности`. Счётчики — это `COUNT(*)`
по таблицам `posts`, `comments`, `deferred_events`, `outbox`.

Дата форматируется через `liveletters_output::format_unix_iso8601_utc`.
Ограничение форматтера: диапазон 1970–2100. Для `created_at > 2_504_889_600`
(что соответствует `2100-01-01 00:00:00 UTC`) функция возвращает
`2100-01-01 00:00:00 UTC` — это сознательное упрощение, не ошибка.
Точность — до секунд; в будущем, если потребуется работа с датами
за пределами 2100, планируется переход на `chrono`.

## Текущее состояние

Реализация готова. Команда читает четыре SQL-счётчика и максимум даты
создания поста; формат вывода стабильный.

## Связанные документы

- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — поставщик методов `count_posts`, `count_comments`, `count_outbox`, `count_deferred_events`, `newest_post_created_at`.
- [`liveletters-output`](../../modules/liveletters-output/INTERFACE.md) — утилита `print_kv`.
