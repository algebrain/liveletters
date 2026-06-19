# `liveletters-lltt-sync` INTERFACE

## Назначение

`liveletters-lltt-sync` — библиотечный крейт, реализующий команду
`lltt sync`:

- `lltt sync` — выполнить полный цикл: сначала `pull`, затем `push`;
- `lltt sync pull` — забрать новые письма с IMAP-сервера текущего
  пользователя liveletters и прогнать их через
  `liveletters_sync::SyncEngine::ingest_batch`;
- `lltt sync push` — отправить каждую запись из таблицы `outbox`
  через SMTP по её полю `delivery`
  ([`OutboxDelivery`](../../modules/liveletters-store/INTERFACE.md#outboxdelivery));
  при успешной отправке запись удаляется из outbox;
- `lltt sync backfill --days=N` — разовая команда: подтянуть письма
  за последние N суток, не сдвигая основной sync-курсор.

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
  необязательной подкомандой `Pull`, `Push` или `Backfill { days }`;
- `SyncAction::{Pull, Push, Backfill { days: u32 }}` — варианты
  подкоманды; `Backfill` по умолчанию берёт `--days=30`;
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
  `subscriptions`, `ResourceFriends` → подписчики, которые есть в
  списке друзей ресурса);
- `run_backfill(ctx, days) -> Result<(), SyncError>` — точка
  входа для подкоманды `backfill` (под признаком `network`);
- `default_profile_id(&str) -> String` — общий хелпер;
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
    /// Разовый заброс: подтянуть письма за последние N суток,
    /// не сдвигая основной sync-курсор.
    Backfill {
        /// Сколько суток заглядывать назад. По умолчанию 30.
        #[arg(long, default_value_t = 30)]
        days: u32,
    },
}

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>>
```

## Требования к запуску

`sync`, `sync pull`, `sync push` и `sync backfill` требуют
заполненной таблицы `mail_settings` (см. `liveletters-settings`). Без
неё команда возвращает
`SyncError::MailSettingsMissing(<profile_id>)` с сообщением
«настройки почты для {profile_id} отсутствуют; запустите
`lltt settings set smtp.host …`».

Минимальный набор ключей:

```
smtp.host, smtp.port, smtp.security, smtp.username, smtp.password, smtp.hello_domain
imap.host, imap.port, imap.security, imap.username, imap.password, imap.mailbox
imap.initial_lookback_days   # см. ниже
```

## Окно первого sync: `imap.initial_lookback_days`

`lltt sync pull` при **самом первом** запуске (когда в `sync_cursors`
ещё нет строки для профиля) запрашивает у IMAP минимальный UID
писем за последние N суток через `UID SEARCH SINCE <дата>` и
начинает с этого UID. N — значение `imap.initial_lookback_days` в
`mail_settings`. По умолчанию `1`, допустимо `0` (с самого начала).

После первого запуска `imap.initial_lookback_days` больше
**не** применяется — sync работает с сохранённого курсора. Чтобы
подтянуть прошлое позже, используйте `lltt sync backfill`.

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

`<N>` — количество успешно отправленных писем (по сумме всех
outbox-записей: для `Direct` — по одному на адрес, для
`ResourceSubscribers` — по одному на подписчика, для `ResourceFriends` —
по одному на друга-подписчика);
`<K>` — количество outbox-записей, при отправке которых произошла
ошибка (запись остаётся в outbox для повторной попытки).

### `lltt sync backfill`

```
получено писем (backfill): <N>
применено:                 <M>
```

`<N>` — количество писем, найденных через `UID SEARCH SINCE` и
скачанных с IMAP. `<M>` — количество писем, успешно прогнанных
через `SyncEngine` (статус `Applied`). Команда **не** сохраняет
новый sync-курсор: после её выполнения `lltt sync pull` продолжает
работать с прежнего места.

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
  позже через подтверждённую подписку).
- `ResourceFriends` — берёт подписчиков ресурса и оставляет только тех,
  кто есть в таблице `friends` владельца ресурса. Используется для
  `friends_only`.

SMTP-ошибка на любом получателе считается ошибкой всей записи:
остальные получатели этой записи не отправляются, запись остаётся
в `outbox`, цикл продолжается со следующей записи.

### `lltt sync`

Печатает сначала отчёт `pull`, затем отчёт `push`. Если `pull`
вернул ошибку, `push` не запускается.
