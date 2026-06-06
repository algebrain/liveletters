# liveletters-lltt-sync

Реализация команды `lltt sync` — сетевой синхронизации текущего
пользователя liveletters с IMAP/SMTP-серверами. Без подкоманды
команда выполняет `pull`, затем `push`; подкоманды `pull` и `push`
оставлены для запуска одной половины цикла.

## Алгоритм `pull`

```
Store::get_mail_settings_record(profile_id)? → MailSettingsRecord
  → ConfiguredImapMailbox::new(ImapMailboxConfig)
Store::get_sync_cursor(profile_id)? → last_seen_uid (или 0)
  → MailboxCursor::from_last_seen_uid
mailbox.fetch_new(&cursor) → FetchBatch { emails, next_cursor }
compute_next_cursor_uid(prev, batch.emails()) → uid.max(prev)
SyncEngine::new(&store).ingest_batch(emails) → SyncReport
tally(&report) → OutcomeCounts
print: получено/применено/дубликатов/некорректных
Store::save_sync_cursor(profile_id, next_uid)
```

Курсор — `last_seen_uid` (максимальный UID из
`message_id = "imap-uid-<N>"` в полученной пачке, ограниченный
снизу предыдущим значением, чтобы не «откатываться» при пустой
пачке). Хранится в таблице `sync_cursors (profile_id PRIMARY KEY,
last_imap_uid INTEGER)`.

## Алгоритм `push`

```
Store::get_mail_settings_record(profile_id)? → MailSettingsRecord
  → ConfiguredSmtpTransport::new(SmtpTransportConfig)
Store::list_outbox_records() → Vec<OutboxRecord>
  для каждой записи:
    record.message_body → decode_message → ProtocolMessage
    Store::list_subscriptions_for_resource(record.resource_id)
    для каждого подписчика:
      build_protocol_email(from, sub.delivery_address, event_type, &msg)
      transport.send(&outgoing)
    при успехе: Store::delete_outbox_record(record.event_id)
print: подключено/отправлено/ошибок
```

Если у записи нет подписчиков — печатается предупреждение, запись
**остаётся** в outbox (это безопасный отказ: подписчики могут
появиться позже).

Если при отправке **хотя бы одному** подписчику произошла ошибка —
запись целиком остаётся в outbox; ошибка печатается в stderr, но
команда завершается с кодом 0 (это сделано осознанно: частичная
отправка считается «неполной», повторная попытка безопасна).

## Алгоритм `sync`

```
pull_dispatch(ctx)?
push_dispatch(ctx)
```

Если `pull` возвращает ошибку, `push` не вызывается. Это важно:
при неудачном получении нельзя создавать у пользователя впечатление,
что полный обмен прошёл успешно.

## Структура таблицы `sync_cursors`

```sql
CREATE TABLE sync_cursors (
    profile_id TEXT PRIMARY KEY,
    last_imap_uid INTEGER NOT NULL
);
```

## Структура таблицы `outbox` (расширение не требуется)

Команда `push` использует существующую таблицу `outbox`
(`event_id, event_type, resource_id, message_body`), где
`message_body` — JSON-представление `ProtocolMessage`
(сериализация через `liveletters_protocol::encode_message`).

## Признак `network`

Крейт `livelettes-lltt-sync` экспортирует `feature = "network"`,
которая включает:

- зависимость `liveletters-mail` с собственным признаком `network`
  (подключает `native-tls`);
- модули `pull` и `push` (реальная IMAP/SMTP-логика);
- варианты `SyncError::Imap`, `SyncError::Smtp`, `parse_security`
  (видны только при наличии признака).

Без `network` команда возвращает
`run::NetworkFeatureDisabled` с подсказкой про сборку. `apps/lltt`
включает признак через `features = ["network"]` в зависимости
`liveletters-lltt-sync`.

## Тесты

- `tests/pull.rs` — 5 юнит-тестов: разбор `MailSecurity`,
  пересчёт курсора (вперёд/на пустой пачке/с мусорными
  message_id), подсчёт исходов `SyncReport`.
- `tests/push.rs` — 4 юнит-теста с реальной SMTP-фикстурой
  (`std::net::TcpListener` на `127.0.0.1:0`): отправка по
  подписчикам, пропуск при пустом списке, проброс ошибки
  SMTP-сервера, round-trip `build_protocol_email`.
- `apps/lltt/tests/cli_sync_pull_push.rs` — 4 e2e-теста: pull
  без настроек (понятная ошибка), pull с идемпотентностью
  курсора, push с очисткой outbox и проверкой `RCPT TO` на
  SMTP-сервере, полный цикл `sync` без подкоманды.
