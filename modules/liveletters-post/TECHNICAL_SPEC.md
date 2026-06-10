# liveletters-post

## Назначение

`liveletters-post` — командный крейт `lltt post`. Создаёт запись в блоге текущего пользователя liveletters и кладёт событие `post_created` в `outbox`. ID и временная метка подбираются автоматически; пользователь задаёт только тело и (опционально) уровень видимости.

## Зона ответственности

- Разбор `Args` (clap-деривация).
- Чтение тела из файла или stdin.
- Проверка, что уровень видимости ∈ {`public`, `friends_only`}.
- Подстановка `post_id`, `created_at`, `resource_id` (= `mail.publish`), `author_id` (= `mail.publish`).
- Делегирование записи в `AppCore::create_post_from_identity`.
- Печать короткого подтверждения.

## Что модуль не должен делать

- знать про clap-дерево всего бинаря `lltt`;
- зависеть от других командных крейтов (`liveletters-comment`, `liveletters-thread`, `liveletters-outbox`);
- генерировать `post_id` вручную (только через `liveletters_app_core::new_post_id`);
- выводить тело записи (только ID);
- изменять `Store` после возврата `Ok(())` (никакой фоновой отправки).

## Граничные случаи

| Случай                                          | Поведение                                                                                |
|-------------------------------------------------|------------------------------------------------------------------------------------------|
| Нет `--body-file`, stdin пуст                   | `PostError::EmptyBody`                                                                    |
| `--body-file` указывает на несуществующий путь  | `PostError::BodyFileNotFound`                                                             |
| `--visibility` ∈ {`public`, `friends_only`}     | Ок, запись сохраняется с этой видимостью                                                 |
| `--visibility` ∈ {`members_only`, `private_community`, …} | `PostError::UnknownVisibility`                                              |
| `LIVELETTERS_HOME`/`current-user` указывают на чужую идентичность | `liveletters_config::load_identity` вернёт `ConfigError`; всплывает как `Config(...)` |
| БД не инициализирована (`lltt init` не запускался) | `liveletters_store::StoreError` из `open_for_home_dir`                                 |
| Одновременный `post new` двух процессов         | Коллизия `post_id` маловероятна при ms-разрешении; разрешается на уровне БД (ошибка SQL) |

## Решения, которые могут поменяться

- `new_post_id` через `unix_millis` — при росте нагрузки стоит заменить на UUIDv7 или счётчик.
- Чтение `stdin` синхронное через `io::stdin().lock()` — допустимо для CLI; для сервера нужна async.
- Допустимые уровни видимости: в текущей версии только `public` и `friends_only`. `members_only` и `private_community` — отдельная задача.

## Текущее минимальное состояние реализации

- `Args { action: PostAction::New(NewArgs { body_file, visibility }) }`.
- `run` делегирует в `AppCore::create_post_from_identity` (через `liveletters-app-core`).
- `print_created` пишет `запись создана: <id>` в stdout.
- Утилиты `read_body` и `parse_visibility` живут в `liveletters-output` и переиспользуются.
- 1 unit + 4 integration + 2 e2e (через бинарь) тестов.

## Критерии готовности

- `cargo build -p liveletters-post` зелёный.
- `cargo test -p liveletters-post` зелёный (1 unit + 4 integration).
- `cargo test -p lltt --test cli_post` зелёный (2 теста).
- `cargo clippy -p liveletters-post --no-deps -- -D warnings` без замечаний.
