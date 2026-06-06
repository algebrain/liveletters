# `liveletters-feed` — INTERFACE

## Назначение

`liveletters-feed` реализует команду:

```sh
lltt feed
lltt feed --limit 20
```

Это лента подписок: посты ресурсов, на которые подписан текущий пользователь liveletters. Собственные посты смотрят через `lltt cu posts`.

## Интерфейс

- crate: `liveletters-feed`
- команда: `feed`
- точка входа: `run(&CommandContext, &Args)`

Экспортируются:

- `Args { limit: Option<usize> }`;
- `FeedError`;
- `print_feed`;
- `run`;
- `CommandContext`;
- `COMMAND_NAME = "feed"`;
- `summary()`, `crate_name()`.

## Поведение

`run`:

1. загружает текущую идентичность;
2. берёт `identity.subscriptions()`;
3. открывает `Store`;
4. берёт `Store::list_posts()`;
5. оставляет только посты, чей `resource_id` входит в подписки и не входит в `resources_owned`;
6. печатает результат.

Порядок постов — обратный хронологический, его задаёт `Store::list_posts()`.

## Вывод

```text
лента подписок: Алиса
постов: 3 (показано: 3)
```

Если подходящих постов нет:

```text
(пусто)
```
