# `liveletters-bounce`: распознавание DSN-bounce

## Назначение

DSN-bounce (Delivery Status Notification, RFC 3461–3464) — это
автоматическое уведомление от почтового сервера о недоставленном письме.
В `lltt` используется для сопоставления отказа с нашим исходящим
`SubscriptionRequested`: если `alice@example.org` не существует, наш
запрос подписки получает bounce, и мы должны убрать `alice` из
`pending_subscriptions` и сообщить пользователю.

Главная задача модуля — **распознать «наши» bounce**, а не любые
письма от `MAILER-DAEMON` (которого вообще нет как стандартного
отправителя, а строки типа «Mail delivery failed» локализуются
у разных провайдеров).

## Что считается «нашим» bounce

Письмо должно удовлетворять **обоим** условиям:

1. `Content-Type: multipart/report; report-type=delivery-status` в
   заголовках. Это стандартный DSN, не ARF (RFC 5965, жалобы на
   спам), не обычное письмо.
2. Внутри `message/delivery-status` секции должно быть `Action: failed`
   (или другое значение) и `Final-Recipient: rfc822; <addr>`.
3. Должен быть `Original-Message-ID: <event_id>@<domain>` — для
   сопоставления с нашим исходящим в `outbox` по `Message-ID`.

Если хотя бы одно условие не выполнено — `parse_dsn` возвращает
`Ok(None)`, и sync-контур игнорирует письмо.

## Публичный API

### `BounceReport`

```rust
pub struct BounceReport {
    pub action: BounceAction,
    pub status: String,
    pub final_recipient: String,
    pub diagnostic_code: String,
    pub original_message_id: Option<String>,
}

pub enum BounceAction {
    Failed,
    Delayed,
    Delivered,
    Relayed,
    Expanded,
    Other(String),
}
```

### `parse_dsn(raw_email: &str) -> Result<Option<BounceReport>, BounceError>`

Распознать и распарсить DSN-bounce в сыром MIME-сообщении. Возвращает
`Ok(None)`, если письмо не DSN-bounce.

Возвращает `Err(BounceError::MissingHeader)` только если у письма нет
заголовка `Content-Type` (что маловероятно, но возможно для битых
тестовых сообщений).

## Использование

`liveletters-sync::SyncEngine::ingest_one` сначала пытается распарсить
письмо как `ProtocolMessage`. Если это не удалось — пробует
`parse_dsn`. Если `Some` — обрабатывает bounce:

1. Извлекает `original_message_id`.
2. Ищет в `outbox` запись с этим `Message-ID`.
3. Если нашёл и `event_type == "subscription_requested"` — удаляет
   `pending_subscriptions` для текущего профиля, сохраняет
   `bounce_records` запись, логирует warning.
4. Если нашёл и `event_type` другой (`post_created`, и т. п.) —
   сейчас только логирует; полная обработка в будущем.

## Что модуль не делает

- Не отправляет почту, не парсит `ProtocolMessage`.
- Не обрабатывает `multipart/alternative` части DSN.
- Не делает различие между `Action: delayed` и `Action: failed` для
  отчётов — сейчас всё, что не `failed`, проходит как `bounced`
  для счётчика, но не приводит к удалению `pending`.
- Не поддерживает `message/global` или `message/global-delivery-status`.

## Соседи

- [`liveletters-protocol`](../liveletters-protocol/INTERFACE.md) — wire-формат `DomainEventPayload`.
- [`liveletters-sync`](../liveletters-sync/INTERFACE.md) — `SyncEngine`, обработка bounce.
- [`liveletters-store`](../liveletters-store/INTERFACE.md) — таблицы `bounce_records`, `outbox` с `Message-ID`.

## Тесты

- `tests/dsn_parse.rs::parses_standard_dsn_5_1_1` — стандартный DSN с `5.1.1`.
- `tests/dsn_parse.rs::ignores_arf_feedback_report` — ARF (`report-type=feedback`) не считается DSN.
- `tests/dsn_parse.rs::ignores_plain_email` — обычное письмо не считается DSN.
