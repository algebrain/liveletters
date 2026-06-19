# Модули LiveLetters

Рабочее пространство `liveletters2` состоит из набора крейтов.

Для каждого крейта ниже приведено краткое описание его назначения. Где есть отдельные документы, рядом даны ссылки на `INTERFACE.md` с публичной поверхностью и `TECHNICAL_SPEC.md` с архитектурой и обоснованиями.

## Базовые библиотеки

#### [`liveletters-domain`](./liveletters-domain/)

Описывает ядро предметной области: идентификаторы аккаунтов и ресурсов, посты, комментарии, события. Не зависит от других крейтов и задаёт только структуру данных, без логики хранения или передачи.

- [INTERFACE.md](./liveletters-domain/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-domain/TECHNICAL_SPEC.md)

#### [`liveletters-utils`](./liveletters-utils/)

Общие утилиты без привязки к хранению, почте или командам: разбор протокольной идентичности вида `Имя <email>`, нормализация адресов, работа с текстом и временем. Используется там, где нужно единообразное поведение в разных частях проекта.

- [INTERFACE.md](./liveletters-utils/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-utils/TECHNICAL_SPEC.md)

#### [`liveletters-store`](./liveletters-store/)

Хранилище данных на SQLite: схема, миграции, операции над записями постов, комментариев, сырых сообщений и отложенных событий. Задаёт также стандартное размещение файлов через `StorePaths` и чистую функцию `resolve_data_dir` для разрешения домашнего каталога.

- [INTERFACE.md](./liveletters-store/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-store/TECHNICAL_SPEC.md)

#### [`liveletters-config`](./liveletters-config/)

Чтение и запись конфигурации: глобального `config.toml` в корне домашнего каталога и структуры `IdentityConfig` для разбора TOML-черновиков идентичностей. Содержит типы почтовых настроек, метаданных и подписок.

- [INTERFACE.md](./liveletters-config/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-config/TECHNICAL_SPEC.md)

#### [`liveletters-secret-box`](./liveletters-secret-box/)

Криптографическая обёртка для локальной защиты секретов. Шифрует пароли почтовых учётных записей через XChaCha20-Poly1305 с ключом в отдельном файле, чтобы они не лежали в базе открытым текстом.

- [INTERFACE.md](./liveletters-secret-box/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-secret-box/TECHNICAL_SPEC.md)

#### [`liveletters-mime`](./liveletters-mime/)

Разбор и сборка MIME-сообщений. Используется при обработке входящих протокольных писем и формировании исходящих, включая поддержку составных частей и заголовков.

- [INTERFACE.md](./liveletters-mime/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-mime/TECHNICAL_SPEC.md)

#### [`liveletters-mail`](./liveletters-mail/)

Почтовый транспорт. Содержит реальные SMTP-отправку и IMAP-получение через сетевые сокеты, без подделок в памяти. Сетевые части подключаются по признаку компиляции.

- [INTERFACE.md](./liveletters-mail/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-mail/TECHNICAL_SPEC.md)

#### [`liveletters-bounce`](./liveletters-bounce/)

Распознаёт автоматические почтовые уведомления о недоставке. Помогает сопоставлять отказ доставки с исходящим запросом подписки и переводить такую подписку из ожидания в состояние ошибки.

- [INTERFACE.md](./liveletters-bounce/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-bounce/TECHNICAL_SPEC.md)

#### [`liveletters-protocol`](./liveletters-protocol/)

Реализация протокола обмена сообщениями между узлами LiveLetters: схема JSON-сообщений, сериализация событий, валидация конвертов. Не зависит от транспорта.

- [INTERFACE.md](./liveletters-protocol/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-protocol/TECHNICAL_SPEC.md)

#### [`liveletters-sync`](./liveletters-sync/)

Движок синхронизации. Принимает сырые входящие сообщения, устраняет дубликаты, откладывает обработку сложных событий и формирует отчёт `SyncReport` с категориями «применено», «отложено», «отклонено».

- [INTERFACE.md](./liveletters-sync/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-sync/TECHNICAL_SPEC.md)

#### [`liveletters-app-core`](./liveletters-app-core/)

Координатор пользовательских сценариев. Сводит вместе `liveletters-domain`, `liveletters-store` и `liveletters-sync` в одну точку входа, через которую верхние слои вызывают операции чтения и записи.

- [INTERFACE.md](./liveletters-app-core/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-app-core/TECHNICAL_SPEC.md)

#### [`liveletters-diagnostics`](./liveletters-diagnostics/)

Набор проверок состояния системы: целостность базы, наличие ключевого файла, валидность конфигов, доступность почтовых серверов. Используется командой `lltt doctor` для формирования сводного отчёта.

- [INTERFACE.md](./liveletters-diagnostics/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-diagnostics/TECHNICAL_SPEC.md)

#### [`liveletters-i18n`](./liveletters-i18n/)

Локализация интерфейса. Хранит шаблоны строк на русском и английском языках для всех типов событий (subject и human_readable_body), подставляет переменные через `%name%`. Определяет `Locale::Ru`/`Locale::En` и `detect_system_locale`.

- [INTERFACE.md](./liveletters-i18n/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-i18n/TECHNICAL_SPEC.md)

#### [`liveletters-log`](./liveletters-log/)

Слой журналирования. Предоставляет функции `log_info`, `log_warn`, `log_error` для structured-логов в JSON-формате с временными метками. Не зависит от других крейтов, кроме `liveletters-config` для настроек ротации.

- [INTERFACE.md](./liveletters-log/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-log/TECHNICAL_SPEC.md)

## Команды CLI

#### [`liveletters-init`](./liveletters-init/)

Команда `lltt init`. Создаёт домашний каталог со всеми подкаталогами, файл базы, ключ обфускации и дефолтную идентичность. Идемпотентна: повторный запуск требует пустой каталог или флаг `--force`.

- [INTERFACE.md](./liveletters-init/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-init/TECHNICAL_SPEC.md)

#### [`liveletters-cu`](./liveletters-cu/)

Команда `lltt cu`. Управляет текущей идентичностью: переключение, просмотр текущего пользователя и показ собственных постов через `lltt cu posts`.

- [INTERFACE.md](./liveletters-cu/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-cu/TECHNICAL_SPEC.md)

#### [`liveletters-sub`](./liveletters-sub/)

Команда `lltt sub`. Управляет подписками текущего пользователя liveletters на блоги других пользователей: запрос подписки, просмотр подтверждённых и ожидающих подписок, отмена ожидания и отписка.

- [INTERFACE.md](./liveletters-sub/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-sub/TECHNICAL_SPEC.md)

#### [`liveletters-friend`](./liveletters-friend/)

Команда `lltt friend`. Добавляет адрес в список друзей текущего пользователя: друзья могут видеть записи `friends_only`, если они подписаны на ресурс владельца.

- [INTERFACE.md](./liveletters-friend/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-friend/TECHNICAL_SPEC.md)

#### [`liveletters-feed`](./liveletters-feed/)

Команда `lltt feed`. Выводит ленту подписок текущего пользователя liveletters: посты ресурсов, на которые он подписан. Поддерживает `--limit <N>`.

- [INTERFACE.md](./liveletters-feed/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-feed/TECHNICAL_SPEC.md)

#### [`liveletters-posts`](./liveletters-posts/)

Команда `lltt cu posts`. Выводит собственные посты текущего пользователя liveletters в обратном хронологическом порядке. Поддерживает `--limit <N>`.

- [INTERFACE.md](./liveletters-posts/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-posts/TECHNICAL_SPEC.md)

#### [`liveletters-inbox`](./liveletters-inbox/)

Команда `lltt inbox`. Реализована подкоманда `import <файл…>`: читает `.eml`-файлы, прогоняет через `SyncEngine::ingest_batch` и печатает сводный отчёт по категориям (применено, дубликат, отложено, отфильтровано, отклонено). Подкоманды `list` и `show` запланированы.

- [INTERFACE.md](./liveletters-inbox/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-inbox/TECHNICAL_SPEC.md)

#### [`liveletters-post`](./liveletters-post/)

Команда `lltt post`. Создаёт новую запись в блоге текущего пользователя liveletters.

- [INTERFACE.md](./liveletters-post/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-post/TECHNICAL_SPEC.md)

#### [`liveletters-comment`](./liveletters-comment/)

Команда `lltt comment`. Создаёт комментарий к существующему посту.

- [INTERFACE.md](./liveletters-comment/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-comment/TECHNICAL_SPEC.md)

#### [`liveletters-outbox`](./liveletters-outbox/)

Команда `lltt outbox`. Показывает очередь исходящих сообщений, ожидающих отправки.

- [INTERFACE.md](./liveletters-outbox/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-outbox/TECHNICAL_SPEC.md)

#### [`liveletters-thread`](./liveletters-thread/)

Команда `lltt thread`. Выводит дерево комментариев к указанному посту.

- [INTERFACE.md](./liveletters-thread/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-thread/TECHNICAL_SPEC.md)

#### [`liveletters-status`](./liveletters-status/)

Команда `lltt status`. Краткий отчёт о состоянии системы: имя текущей идентичности, число постов, число непрочитанных входящих сообщений.

- [INTERFACE.md](./liveletters-status/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-status/TECHNICAL_SPEC.md)

#### [`liveletters-doctor`](./liveletters-doctor/)

Команда `lltt doctor`. Запускает полную диагностику через `liveletters-diagnostics` и печатает сводный отчёт по всем проверкам.

- [INTERFACE.md](./liveletters-doctor/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-doctor/TECHNICAL_SPEC.md)

#### [`liveletters-settings`](./liveletters-settings/)

Команда `lltt settings`. Показывает и изменяет пользовательские настройки: SMTP/IMAP, язык интерфейса, параметры профиля.

- [INTERFACE.md](./liveletters-settings/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-settings/TECHNICAL_SPEC.md)

#### [`liveletters-lltt-sync`](./liveletters-lltt-sync/)

Команда `lltt sync`. Запускает сетевую синхронизацию с почтовым сервером: без подкоманды выполняет получение, затем отправку; `sync pull` и `sync push` запускают одну половину цикла. Не путать с библиотекой `liveletters-sync` — это разные крейты: первый — оболочка команды, второй — сам движок.

- [INTERFACE.md](./liveletters-lltt-sync/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-lltt-sync/TECHNICAL_SPEC.md)

## Утилиты вывода

#### [`liveletters-output`](./liveletters-output/)

Общие функции человекочитаемого вывода для команд `lltt`. Содержит единую точку маскирования секретов, печать пар «ключ-значение», таблиц с выравниванием колонок и полной идентичности с разбиением на секции. Не выполняет бизнес-логики и не обращается к базе.

- [INTERFACE.md](./liveletters-output/INTERFACE.md)
- [TECHNICAL_SPEC.md](./liveletters-output/TECHNICAL_SPEC.md)
