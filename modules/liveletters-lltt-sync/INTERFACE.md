# `liveletters-lltt-sync` INTERFACE

## Назначение

`liveletters-lltt-sync` — библиотечный крейт, реализующий команду
`lltt sync`:

- `lltt sync` — выполнить полный цикл: сначала `pull`, затем `push`;
- `lltt sync pull` — забрать новые письма с IMAP-сервера текущего
  пользователя liveletters и прогнать их через
  `liveletters-sync::SyncEngine::ingest_batch`;
- `lltt sync push` — отправить каждую запись из таблицы `outbox`
  через SMTP по её полю `delivery`
  ([`OutboxDelivery`](../../modules/liveletters-store/INTERFACE.md#outboxdelivery));
  при успешной отправке запись удаляется из outbox.

Реальная реализация подключается под признаком `network` (см.
`Cargo.toml`). Без признака команда возвращает
`run::NetworkFeatureDisabled` с понятным сообщением; сборка
`apps/lltt` сама включает признак, поэтому пользователь видит
нормальное поведение по умолчанию.

## Где находится интерфейс

- crate: `liveletters-lltt-sync`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- `Args { action: Option<SyncAction> }` — clap-аргументы с
  необязательной подкомандой `Pull` или `Push`;
- `SyncAction::{Pull, Push}` — варианты подкоманды;
- `SyncError` — типизированные ошибки команды
  (`MailSettingsMissing`, `Imap`, `Smtp`, `Store`, `Engine`,
  `Protocol`, `OutboxDecode`, `UnknownMailSecurity`);
- `run(ctx, args) -> Result<(), Box<dyn Error + Send + Sync>>` —
  единая точка запуска;
- `OutcomeCounts { applied, duplicates, malformed }` — счётчики
  исходов `SyncReport` (для `pull`);
- `compute_next_cursor_uid(prev, &[ReceivedEmail]) -> u64` —
  пересчёт курсора IMAP;
- `parse_security(&str) -> Result<MailSecurity, SyncError>` —
  разбор строкового значения `smtp.security` / `imap.security`;
  принимает `ssl`/`SSL` как синоним `tls`;
- `tally(&SyncReport) -> OutcomeCounts` — подсчёт исходов;
- `send_outbox_record(&Store, &ConfiguredSmtpTransport, &str, &OutboxRecord) -> Result<usize, SyncError>` —
  низкоуровневая отправка одной outbox-записи (для `push`);
  получатели определяются полем `OutboxRecord.delivery`
  (`Direct` → конкретные адреса, `ResourceSubscribers` → таблица
  `subscriptions`);
- `CommandContext` (реэкспорт из `liveletters-output`);
- константы `COMMAND_NAME`, `summary()`, `crate_name()`.

## Сигнатура запуска

```rust
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<SyncAction>,
}

#[derive(Debug, clap::Subcommand)]
pub enum SyncAction {
    /// Забрать новые письма с IMAP и прогнать через SyncEngine.
    Pull,
    /// Отправить исходящие из outbox через SMTP.
    Push,
}

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>>
```

## Требования к запуску

`sync`, `sync pull` и `sync push` требуют заполненной таблицы
`mail_settings` (см. `liveletters-settings`). Без неё команда возвращает
`SyncError::MailSettingsMissing(<profile_id>)` с сообщением
«настройки почты для {profile_id} отсутствуют; запустите
`lltt settings set smtp.host …`».

Минимальный набор ключей:

```
smtp.host, smtp.port, smtp.security, smtp.username, smtp.password, smtp.hello_domain
imap.host, imap.port, imap.security, imap.username, imap.password, imap.mailbox
```

## Формат вывода

### `lltt sync pull`

```
получено писем:       <N>
применено событий:    <M>
дубликатов:           <K>
некорректных писем:   <L>
```

`<N>` — количество писем, отданных IMAP-сервером за одну сессию
(с `UID > last_seen_uid` и заголовком `X-LiveLetters-Protocol: v1`);
обычные письма не скачиваются целиком. `<M>` / `<K>` / `<L>` — подсчёт
по исходам `SyncReport` (`Applied` / `Duplicate` / `Malformed`).

### `lltt sync push`

```
подключено к <smtp.host>
отправлено писем:     <N>
ошибок отправки:      <K>
```

`<N>` — количество успешно отправленных писем (по сумме
всех outbox-записей: для `Direct` — по одному на адрес, для
`ResourceSubscribers` — по одному на подписчика);
`<K>` — количество outbox-записей, при отправке которых произошла
ошибка (запись остаётся в outbox для повторной попытки).

### Поведение `push`

`sync push` не вычисляет адресацию. Для каждой записи из `outbox`
он смотрит на её `OutboxDelivery`:

- `Direct([a, b, c])` — отправляет письмо по адресу `a`, затем `b`,
  затем `c`. Каждый адрес — отдельное SMTP-сообщение; получатели
  не объединяются.
- `ResourceSubscribers` — берёт всех подписчиков из таблицы
  `subscriptions` для заданного `resource_id` и отправляет по
  письму каждому. Если подписчиков нет, запись остаётся в `outbox`
  и будет обработана следующим `push` (подписчики могут появиться
  позже через `subscription_changed`).

SMTP-ошибка на любом получателе считается ошибкой всей записи:
остальные получатели этой записи не отправляются, запись остаётся
в `outbox`, цикл продолжается со следующей записи.

### `lltt sync`

Печатает сначала отчёт `pull`, затем отчёт `push`. Если `pull`
вернул ошибку, `push` не запускается.
