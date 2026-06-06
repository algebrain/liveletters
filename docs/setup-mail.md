# Настройка почты для `lltt sync`

Команда `lltt sync` использует настройки `mail_settings` из локальной
БД: сначала получает новые письма с IMAP, затем отправляет исходящие
через SMTP. Основной способ заполнить настройки — указать почту в
черновике из `lltt user init <имя>` и добавить пользователя через
`lltt user add`.

## Основной способ

Для полного цикла `lltt sync` нужны оба блока: IMAP для получения и
SMTP для отправки.

```sh
lltt user init alice
```

Отредактируйте созданный файл `drafts/alice.toml`:

```toml
[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "alice@example.org"
password = "пароль"
pwd_obfuscate = true
hello_domain = "example.org"

[mail.imap]
host = "imap.example.org"
port = 993
security = "tls"
username = "alice@example.org"
password = "пароль"
pwd_obfuscate = true
mailbox = "INBOX"
```

Затем добавьте пользователя:

```sh
lltt user add alice
lltt cu alice
```

Если пароль непустой и `pwd_obfuscate = true`, команда отдельно
попросит подтвердить SMTP- и IMAP-пароль. Ввод скрытый: на экране
видны только звёздочки. После сохранения открытый пароль в TOML
заменяется значением `obf:v1:...`, а настройки копируются в
`mail_settings`.

## Точечная правка

`lltt settings set` нужен, когда нужно изменить одно поле уже после
добавления пользователя:

```sh
lltt settings set smtp.host       smtp.example.org
lltt settings set imap.mailbox    INBOX
```

Допустимые значения `*.security`: `none`, `starttls`, `tls`, `ssl`.
Значение `ssl` или `SSL` означает то же, что `tls`: защищённое
соединение сразу при подключении.
Пароль при сохранении через `settings set` также сохраняется в скрытом
виде в БД.

## Что делает `lltt sync pull`

1. Читает `mail_settings` и `sync_cursors.last_imap_uid`.
2. Подключается к IMAP-серверу, входит в учётную запись, открывает почтовый ящик.
3. Делает `UID SEARCH UID <last+1>:*` и для каждого нового UID —
   `UID FETCH BODY.PEEK[]`.
4. Прогоняет полученные письма через `liveletters-sync::SyncEngine::ingest_batch`.
5. Сохраняет максимальный UID обратно в `sync_cursors`.

Повторный `pull` сразу после успешного получает 0 новых писем —
идемпотентность гарантируется сохранённым курсором.

## Что делает `lltt sync push`

1. Читает `mail_settings` и все записи из `outbox`.
2. Для каждой записи:
   - декодирует `message_body` в `ProtocolMessage`;
   - ищет подписчиков ресурса в таблице `subscriptions`;
   - для каждого подписчика собирает `OutgoingEmail` и отправляет
     через SMTP;
   - при успехе на **всех** подписчиках удаляет запись из `outbox`.
3. Если у ресурса нет подписчиков — печатает предупреждение и
   оставляет запись (безопасный отказ: подписчики могут появиться
   позже).

## Что делает `lltt sync`

`lltt sync` без подкоманды выполняет `lltt sync pull`, затем
`lltt sync push`. Если получение завершилось ошибкой, отправка не
запускается.

## Ограничения текущей версии

- **Без ретраев.** Одна неудачная отправка — запись остаётся в
  outbox. Автоматический повтор с back-off — в планах.
- **Один SMTP/IMAP-профиль.** Несколько почтовых ящиков — в планах.
- **TLS без кастомных CA.** `native-tls` со стандартными центрами
  сертификации; самоподписанные сертификаты не поддерживаются.
- **SMTP-пароль не валидируется в момент сохранения.** Спецсимволы
  (`\`, `"`) ломают IMAP-`LOGIN`; используйте пароль без них.

## Проверка

После настройки:

```sh
lltt doctor       # базовая диагностика
lltt sync         # получение с IMAP, затем отправка исходящих
lltt sync pull    # только попытка IMAP-сеанса
lltt sync push    # только отправка исходящих
```

Если `pull` или `push` завершается с ошибкой — прочитайте
сообщение: в нём указан класс (`MailSettingsMissing` / `Imap` /
`Smtp` / `Store` / `Engine` / `Protocol` / `OutboxDecode`).
