# Бинарь `lltt` — INTERFACE

## Назначение

`lltt` — основной CLI проекта LiveLetters. Это тонкая оболочка, которая:

1. разбирает аргументы командной строки через `clap`;
2. строит [`CommandContext`](../../modules/liveletters-output/src/context.rs) (общий домашний каталог, каталог состояния текущего пользователя и имя текущего пользователя liveletters, если он требуется команде);
3. инициализирует глобальный логгер из `GlobalConfig.log` (по умолчанию выключен);
4. диспетчеризует вызов в `run(...)` соответствующего командного крейта из [`modules/`](../../modules);
5. вызывает `liveletters_log::shutdown()` перед выходом (сбрасывает буфер).

Содержательной логики в самом `apps/lltt` нет — она разнесена по независимым крейтам (`liveletters-init`, `liveletters-cu`, `liveletters-sub`, `liveletters-friend`, `liveletters-feed`, `liveletters-inbox`, `liveletters-post`, `liveletters-comment`, `liveletters-outbox`, `liveletters-thread`, `liveletters-status`, `liveletters-doctor`, `liveletters-settings`, `liveletters-lltt-sync`, плюс общий `liveletters-output`). Каждый из них реализует свою подкоманду и документирован отдельно.

## Где находится интерфейс

- бинарь: `apps/lltt`
- точка входа: `src/main.rs`

Наружу бинарь экспортирует только имя `lltt` (имя бинаря в `Cargo.toml`).

## Подкоманды (clap-дерево)

Каждая подкоманда имеет собственные `Args` в своём крейте.

| Подкоманда | Крейт | Что делает |
|---|---|---|
| `init` | `liveletters-init` | Создаёт домашний каталог (`LIVELETTERS_HOME` или `<HOME>/.liveletters/`) со служебными каталогами, файлом БД и ключом скрытия паролей. Не создаёт пользователя по умолчанию. `--force` разрешает инициализацию непустого каталога без удаления существующих файлов. |
| `cu` | `liveletters-cu` | Показать, выбрать или просмотреть текущего пользователя liveletters. См. раздел «Подкоманды `cu`» ниже. |
| `user` | `liveletters-cu` | Управление списком идентичностей: создать черновик, добавить, показать, удалить, перечислить. См. раздел «Подкоманды `user`» ниже. |
| `sub` | `liveletters-sub` | Управление подписками текущего пользователя liveletters на блоги других пользователей. См. раздел «Подкоманды `sub`» ниже. |
| `friend` | `liveletters-friend` | Добавить адрес в друзья, чтобы он мог видеть записи `friends_only` при наличии подписки на ваш ресурс. |
| `feed` | `liveletters-feed` | Показать ленту подписок текущего пользователя liveletters: посты ресурсов, на которые он подписан. Поддерживает `--limit <N>` (показать не более N последних постов подписок). |
| `inbox` | `liveletters-inbox` | Управление входящей почтой. Подкоманды: `import <файл…>` — импортировать одно или несколько писем из `.eml`-файлов через `SyncEngine`; `list [--status <категория>] [--limit <N>]` — таблица последних N строк `raw_messages` с фильтром по статусу; `show <message_id>` — печать полного тела одного письма. |
| `post` | `liveletters-post` | Создать запись в блоге. Подкоманда: `new [--body-file <path>] [--visibility <public\|friends_only>]` — тело из файла или из stdin, видимость по умолчанию `public`. |
| `comment` | `liveletters-comment` | Создать комментарий к записи. Подкоманда: `new --post <id> [--parent <id>] [--body-file <path>]`. Видимость наследуется от исходной записи. |
| `outbox` | `liveletters-outbox` | Показать исходящую очередь (read-only). Подкоманда: `list` — таблица `event_id \| event_type \| resource_id \| delivery`. |
| `thread` | `liveletters-thread` | Показать обсуждение (запись + дерево комментариев). Использование: `lltt thread <post_id>`. |
| `status` | `liveletters-status` | Краткий отчёт: 5 полей (постов, комментариев, отложенных, исходящих, последняя активность). |
| `doctor` | `liveletters-doctor` | Полная диагностика синхронизации: 9 полей (здоровье + 7 категорий входящих + размер outbox). С флагом `--verbose` (`-v`) дополнительно показывает deferred-события, identities и размеры таблиц БД. |
| `settings` | `liveletters-settings` | Показать или изменить настройки `user_settings`/`mail_settings` (БД) и `GlobalConfig.log` (TOML). Подкоманды: `show` (по умолчанию) — сводка (включая секцию `[логирование]`, если она отличается от дефолта); `set <ключ> <значение>` — изменить одно поле; допустимы `log.destination`, `log.level`, `log.max_size_bytes`, `log.keep_files`, `log.include_bodies` для включения/выключения журнала. |
| `set` | `liveletters-settings` | Сокращение для `settings set <ключ> <значение>` без вложенной подкоманды: `lltt set <ключ> <значение>`. Делает ровно то же, что `lltt settings set`, в том числе для `log.*`. |
| `sync` | `liveletters-lltt-sync` | Сетевая синхронизация. Без подкоманды выполняет `pull`, затем `push`; `sync pull` и `sync push` запускают только одну половину цикла. |

Подкоманды `sync` работают при наличии в `mail_settings` настроек SMTP/IMAP; обычно они попадают туда из почтовых секций черновика при `lltt user add`. Если настроек нет, команда возвращает `SyncError::MailSettingsMissing` (код 1) с подсказкой заполнить почту или запустить `lltt settings set smtp.host …`.

`sync push` отправляет записи из `outbox` строго по их полю `delivery` (см. [`liveletters-lltt-sync::send_outbox_record`](../../modules/liveletters-lltt-sync/src/push.rs)): `Direct([адрес…])` — поштучно на каждый адрес, `ResourceSubscribers` — по одному письму каждому подписчику соответствующего ресурса, `ResourceFriends` — только подписчикам из списка друзей ресурса.

### Подкоманды `cu`

Команда `lltt cu` имеет четыре операции и работает только с выбранным текущим пользователем. Старые формы управления списком (`lltt cu list`, `lltt cu add`, `lltt cu rm`, `lltt cu show <имя>`) запрещены и возвращают ошибку с подсказкой перейти на `lltt user ...`.

| Форма | Что делает |
|---|---|
| `lltt cu <имя>` | Переключает текущего пользователя liveletters на `<имя>`. Имя должно существовать в `<home>/identities/`. Создаёт локальное состояние `<home>/users/<имя>/`, записывает новое значение в `<home>/current-user`. Печатает `текущий пользователь: <имя>`. |
| `lltt cu` | Без аргументов — печатает имя текущего пользователя liveletters (то, что лежит в `<home>/current-user`). |
| `lltt cu show [--reveal]` | Печатает содержимое конфига текущего пользователя через `liveletters_output::print_identity`. Пароли SMTP/IMAP по умолчанию маскируются как `********`; флаг `--reveal` показывает их в открытом виде. |
| `lltt cu posts [--limit <N>]` | Печатает собственные посты текущего пользователя liveletters в обратном хронологическом порядке. Это не лента подписок; лента подписок вызывается через `lltt feed`. |

### Подкоманды `user`

Команда `lltt user` управляет каталогом `<home>/identities/` и черновиками `<home>/drafts/`. Она не требует выбранного текущего пользователя, поэтому работает сразу после `lltt init`.

| Форма | Что делает |
|---|---|
| `lltt user list` | Печатает построчно имена всех конфигов в `<home>/identities/`. После чистого `init` список пуст. |
| `lltt user init <имя> [--force]` | Создаёт черновик `<home>/drafts/<имя>.toml`, печатает путь и содержимое. Без `--force` не перезаписывает существующий черновик. |
| `lltt user show <имя> [--reveal]` | Печатает содержимое конфига `<home>/identities/<имя>.toml`. Пароли маскируются, кроме режима `--reveal`. |
| `lltt user add <имя> [--from <путь>]` | Читает TOML-файл, валидирует имя и содержимое, сохраняет `<home>/identities/<имя>.toml`, копирует почтовые секции в `mail_settings` пользовательского состояния `<home>/users/<имя>/`. Без `--from` берёт `<home>/drafts/<имя>.toml`. Если включено `pwd_obfuscate`, отдельно просит подтвердить SMTP- и IMAP-пароль скрытым вводом со звёздочками и сохраняет их в виде `obf:v1:...`. Текущего пользователя не меняет. |
| `lltt user rm <имя> --yes` | Удаляет файл `<home>/identities/<имя>.toml`. **Требует флаг `--yes`** для подтверждения. **Запрещено** удалять того, кто сейчас выбран текущим (сначала `lltt cu <другое_имя>`, затем `lltt user rm <имя> --yes`). |

Полный публичный API `liveletters-cu` (включая коды ошибок, формат `Args`, поведение `CuAction`) — в [`modules/liveletters-cu/INTERFACE.md`](../../modules/liveletters-cu/INTERFACE.md).

### Подкоманды `sub`

Команда `lltt sub` управляет подписками текущего пользователя liveletters на блоги (т.е. на `mail.publish` других пользователей). Новая подписка создаёт ожидание подтверждения и исходящее событие `subscription_requested`; после ответа владельца ресурса она становится подтверждённой.

| Форма | Что делает |
|---|---|
| `lltt sub <адрес>` | Запросить подписку на блог по адресу `<адрес>` (внешний `mail.publish` другого пользователя). Адрес проверяется парсером `ResourceAddress::new()`. Печатает, что подписка запрошена и ожидает подтверждения. |
| `lltt sub list` | Печатает секции «подписан на:», «мои подписчики:», «мои друзья:» и «я в друзьях у:». |
| `lltt sub rm <адрес>` | Отписаться. Удаляет ожидающую или подтверждённую подписку и порождает `subscription_revoked`, чтобы владелец блога удалил подписчика у себя. |

Ошибки: невалидный адрес → `SubError::Domain` (код 1); нет `init` или отсутствует домашний каталог → ошибка контекста (код 2); `identities/<текущий>.toml` отсутствует → `SubError::Config(ConfigError::UnknownIdentity)` (код 1); неверное число токенов → `SubError::InvalidArgs` (код 1).

Полный публичный API `liveletters-sub` (включая коды ошибок, формат `Args`, поведение `SubAction`) — в [`modules/liveletters-sub/INTERFACE.md`](../../modules/liveletters-sub/INTERFACE.md).

### Подкоманды `inbox`

Команда `lltt inbox` управляет входящей почтой. Реализованы три подкоманды.

| Форма | Что делает |
|---|---|
| `lltt inbox import <файл.eml> [<файл.eml>…]` | Импортировать одно или несколько писем через `SyncEngine::ingest_batch`; напечатать построчно исход для каждого сообщения и итоговую сводку (`применено / дубликатов / отложено / отфильтровано / отклонено`). |
| `lltt inbox list [--status <категория>] [--limit <N>]` | Показать последние N строк таблицы `raw_messages` (по умолчанию 20, новые сверху). Категории для `--status`: `applied`, `duplicate`, `replay`, `unauthorized`, `invalid`, `malformed`. Без `--status` — все категории. |
| `lltt inbox show <message_id>` | Напечатать полное тело (`raw_message`) одного письма по его `message_id` (значение заголовка `Message-ID`, например `<p-1@example.test>`). При отсутствии — ошибка `InboxError::MessageNotFound` (код 1). |

Полный публичный API `liveletters-inbox` (включая `ALLOWED_STATUSES`, формат таблицы, `preview`-обрезку, `InboxError::InvalidStatus`, `InboxError::MessageNotFound`) — в [`modules/liveletters-inbox/INTERFACE.md`](../../modules/liveletters-inbox/INTERFACE.md).

### Подкоманды `settings`

Команда `lltt settings` показывает или изменяет настройки, хранящиеся в таблицах `user_settings` и `mail_settings` БД (SMTP/IMAP-параметры, никнейм, адрес почты). Слой идентичности (`identities/<name>.toml`) управляется через `lltt user`; `settings` нужен как точечное дополнение после добавления идентичности.

| Форма | Что делает |
|---|---|
| `lltt settings` (или `lltt settings show`) | Печатает содержимое обеих таблиц через `print_kv`. Если запись отсутствует — печатает `[user_settings] отсутствует` с подсказкой использовать `set`. Пароли маскируются как `********` (`--reveal` не предусмотрен). |
| `lltt settings set <ключ> <значение>` | Записывает одно поле. Допустимые 17 ключей: `nickname`, `email_address`, `avatar_url`, `language`, `setup_completed` (поле `user_settings`); `smtp.host`, `smtp.port`, `smtp.security`, `smtp.username`, `smtp.password`, `smtp.hello_domain`, `imap.host`, `imap.port`, `imap.security`, `imap.username`, `imap.password`, `imap.mailbox` (поля `mail_settings`). Ключ `language` принимает `ru` или `en`; иное значение отвергается `SettingsError::InvalidValue`. При первом `set` строка `user_settings` создаётся с `language` из `liveletters_i18n::detect_system_locale()` (см. `LC_ALL`/`LC_MESSAGES`/`LANG`). Пароли (`smtp.password`, `imap.password`) сохраняются в форме `obf:v1:…` через `SecretBox::obfuscate`. |
| `lltt set <ключ> <значение>` | Короткая форма той же команды верхнего уровня. Ведёт себя идентично `lltt settings set`, включая маршрутизацию `log.*` в `<home>/config.toml` и валидацию через `LogConfig::set_field`. |

Ошибки: неизвестный ключ → `SettingsError::InvalidKey` (код 1); неверное число аргументов → `SettingsError::InvalidArgs` (код 1); нет `init` → ошибка контекста (код 2).

Полный публичный API `liveletters-settings` (включая `ALLOWED_KEYS`, обфускацию паролей, `SettingsError`) — в [`modules/liveletters-settings/INTERFACE.md`](../../modules/liveletters-settings/INTERFACE.md).

## Текущий пользователь liveletters

Имя текущего пользователя liveletters хранится в **текстовом файле `<home>/current-user`** (одна строка — имя без расширения). Задаётся только командой `lltt cu <имя>`. Читается командой `lltt cu`, командой `lltt cu show` и всеми остальными командами, которым нужна текущая идентичность.

Локальное состояние текущего пользователя хранится отдельно в `<home>/users/<имя>/`. В этом каталоге лежит его `liveletters.sqlite3`, ключ скрытия паролей, входящие письма, исходящая очередь и курсоры синхронизации. Поэтому два пользователя в одном `LIVELETTERS_HOME` не видят посты друг друга через общую БД: обмен должен пройти через SMTP/IMAP.

Если файл `<home>/current-user` отсутствует, команды `init`, `user ...`, `cu <имя>` и старые запрещённые формы `cu list/add/rm/show <имя>` могут выполниться или вернуть свою обычную ошибку. Команды, которым нужен уже выбранный пользователь (`status`, `feed`, `post`, `cu`, `cu show`, `cu posts` и т. п.), возвращают код 2 и подсказывают: `lltt user init <имя>`, `lltt user add <имя>`, затем `lltt cu <имя>`.

См. также [`liveletters-config::read_current_identity`](../../modules/liveletters-config/src/io.rs) и [`liveletters-config::write_current_identity`](../../modules/liveletters-config/src/io.rs).

## Окружение

| Переменная | Эффект |
|---|---|
| `LIVELETTERS_HOME` | Если задана — путь к домашнему каталогу используется как есть, без суффикса. |
| `HOME` (Unix, MSYS/Cygwin/Git Bash) | Если `LIVELETTERS_HOME` не задана — путь к домашнему каталогу берётся как `<HOME>/.liveletters/`. |
| `USERPROFILE` (нативный Windows) | Аналогично, если `HOME` нет. |

См. также [`liveletters-store::resolve_data_dir_from_env`](../../modules/liveletters-store/src/paths.rs).

## Коды выхода

| Код | Значение |
|---|---|
| `0` (`ExitCode::SUCCESS`) | Команда выполнена успешно. |
| `1` | Команда вернула ошибку. Текст ошибки напечатан в `stderr` (`ошибка: <текст>`). |
| `2` | Ошибка контекста: не удалось разрешить домашний каталог, домашний каталог не существует (и команда не `init`), или отсутствует файл `<home>/current-user` для команды, которой нужен текущий пользователь. Текст ошибки напечатан в `stderr`. |

## Файлы результата

Бинарь `lltt` печатает данные в `stdout` и сообщения об ошибках в `stderr`. Никаких постоянных файлов в произвольных местах он не создаёт: вся работа с файлами делегирована командным крейтам. Общие файлы лежат в домашнем каталоге, а рабочее состояние пользователя — в `<home>/users/<имя>/`.

## Зависимости

В [`apps/lltt/Cargo.toml`](Cargo.toml) бинарь зависит от 14 крейтов-команд, от `liveletters-config`, `liveletters-store`, `liveletters-output` и от `clap` (для разбора аргументов). Полный список — в файле манифеста.

## Пример

```sh
$ export LIVELETTERS_HOME=/var/lib/lltt
$ lltt init
инициализирован /var/lib/lltt
создан lltt.db
создан mail-password-obfuscation.key
создан каталог identities
создан каталог drafts

$ lltt user init alice
создан черновик /var/lib/lltt/drafts/alice.toml
...

$ lltt user add alice
добавлен identities/alice.toml

$ lltt cu alice
текущий пользователь: alice

$ lltt cu
alice

$ lltt cu show
[identity]
display_name: Alice

[mail]
publish: alice-publish@example.org
receive: [0] alice-feed@example.org

$ echo "Привет, мир." | lltt post new
запись создана: post-1717161234567

$ lltt cu posts
посты пользователя
пользователь: Alice
постов: 1

$ lltt thread post-1717161234567
┌─ пост #post-1717161234567 от alice-publish@example.org
│  visibility: public
│  Привет, мир.
└─

комментарии: 0

(нет комментариев)

$ lltt outbox list
неотправленные события: 1

event_id                          event_type        resource_id
post-created:post-1717161234567   post_created      alice-publish@example.org
```
