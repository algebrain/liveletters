# liveletters-comment

## Назначение

`liveletters-comment` — командный крейт `lltt comment`. Добавляет комментарий к записи (с возможностью вложенного ответа через `--parent`) и кладёт событие `comment_created` в `outbox`.

## Зона ответственности

- Разбор `Args` (clap-деривация).
- Чтение тела из файла или stdin.
- Делегирование создания комментария в `AppCore::create_comment_from_identity`.
- Печать ID созданного комментария.

## Что модуль не должен делать

- знать про clap-дерево всего бинаря `lltt`;
- зависеть от других командных крейтов;
- проверять, что `--post` существует (это делает `AppCore` через `PostNotFound`);
- проверять, что `--parent` существует (то же);
- выводить тело комментария.

## Граничные случаи

| Случай                                          | Поведение                                                                                |
|-------------------------------------------------|------------------------------------------------------------------------------------------|
| `--post` указывает на несуществующую запись      | `CommentError::AppCore(AppCoreError::PostNotFound { post_id })`                           |
| `--parent` указывает на несуществующий комментарий | `CommentError::AppCore(AppCoreError::CommentNotFound { ... })` или несоответствие `post_id` в БД |
| `--parent` указывает на комментарий из другого поста | `AppCoreError::Domain(...)` — комментарий-сирота                                          |
| Пустое тело (файл пуст или stdin пуст)          | `CommentError::EmptyBody`                                                                 |
| `--body-file` указывает на несуществующий путь  | `CommentError::BodyFileNotFound`                                                          |
| Передан `--visibility`                          | Ошибка разбора аргументов: у комментариев нет отдельной видимости                          |
| Одновременное создание двух комментариев        | Коллизия `comment_id` маловероятна при ms-разрешении                                      |

## Решения, которые могут поменяться

- `new_comment_id` через `unix_millis` — заменить на UUIDv7 при росте нагрузки.
- Валидация `parent` — сейчас опирается на БД-уровень (`foreign key` или явная проверка). Если БД-уровень ослабнет, нужно добавить явную проверку в `AppCore`.
- Видимость комментария всегда наследуется от исходной записи. Команда
  комментария её не принимает и не вычисляет сама.

## Текущее состояние

- `Args { action: CommentAction::New(NewArgs { post, parent, body_file }) }`.
- `run` делегирует в `AppCore::create_comment_from_identity`.
- 5 integration + 2 e2e тестов.

## Критерии готовности

- `cargo build -p liveletters-comment` зелёный.
- `cargo test -p liveletters-comment` зелёный (5 integration).
- `cargo test -p lltt --test cli_comment` зелёный (2 теста).
- `cargo clippy -p liveletters-comment --no-deps -- -D warnings` без замечаний.
