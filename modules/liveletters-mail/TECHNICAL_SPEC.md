# liveletters-mail

## Назначение

`liveletters-mail` это библиотека почтового транспорта LiveLetters. Она отвечает за IMAP, SMTP, разбор email и низкоуровневое извлечение MIME-структуры.

## Зона ответственности

- SMTP send;
- IMAP fetch;
- mailbox scanning;
- raw email parsing;
- MIME extraction;
- transport-level retries;
- mapping raw email в внутренние транспортные типы.

## Что модуль не должен делать

- применять доменные события;
- решать вопросы авторизации;
- содержать UI-логику;
- знать о materialized views;
- быть местом orchestration сложных use cases.

## Основные подсистемы

- SMTP adapter;
- IMAP adapter;
- raw parser;
- MIME extractor;
- transport retry policy;
- mailbox cursor helpers.

## Требования к API

- получение новых писем;
- отправка письма;
- парсинг сырого письма;
- извлечение технической части и человекочитаемой части;
- транспортные ошибки как отдельные типы;
- абстракции для тестовых transport adapters.

## Требования к структуре каталога

- `src/imap`;
- `src/smtp`;
- `src/parser`;
- `src/mime`;
- `src/errors`;
- `tests/fixtures`;
- `tests`.

## Требования к тестам

Покрытие тестами обязательно.

Минимум:

- tests на parsing email fixtures;
- tests на MIME edge cases;
- tests на transport-level retry behavior;
- tests на чтение multipart сообщений;
- tests на ошибки аутентификации и сетевые ошибки через adapters или mocks.

## Требования к документации

Обязательна документация:

- описание транспортного API;
- поддерживаемые форматы email;
- правила обработки multipart писем;
- ограничения и негарантии transport слоя;
- способ настройки IMAP и SMTP.

## Критерии готовности

- библиотека умеет отправлять и получать email через абстракции;
- raw parsing работает на fixture-наборе;
- ошибки транспорта типизированы;
- библиотека не принимает доменные решения.

## Связанные документы

- `/home/algebrain/src/my/liveletters/docs/idea.technical.md`
- `/home/algebrain/src/my/liveletters/docs/workspace-structure.md`
