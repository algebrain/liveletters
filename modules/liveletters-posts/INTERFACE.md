# `liveletters-posts` — INTERFACE

## Назначение

`liveletters-posts` реализует показ постов текущего пользователя liveletters. В CLI эта логика вызывается через:

```sh
lltt cu posts
lltt cu posts --limit 20
```

Крейт не является лентой подписок. Лента подписок живёт в `liveletters-feed`.

## Интерфейс

- crate: `liveletters-posts`
- команда: `posts` внутри `lltt cu`
- точка входа: `run(&CommandContext, &Args)`

Экспортируются:

- `Args { limit: Option<usize> }`;
- `PostsError`;
- `print_posts`;
- `run`;
- `CommandContext`;
- `COMMAND_NAME = "posts"`;
- `summary()`, `crate_name()`.

## Поведение

`run`:

1. открывает `Store` для домашнего каталога;
2. загружает текущую идентичность из `identities/<current>.toml`;
3. вызывает `get_current_user_posts` с `author_id = identity.account_id()`;
4. печатает посты через `print_posts`.

Посты выводятся в обратном хронологическом порядке. Порядок задаёт `Store::list_posts()`.

## Вывод

Заголовок:

```text
посты пользователя: Алиса
постов: 2 (показано: 2)
```

Если постов нет:

```text
(пусто)
```

`--limit N` ограничивает уже отсортированный список.
