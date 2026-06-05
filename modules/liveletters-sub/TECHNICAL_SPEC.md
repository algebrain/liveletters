# Крейт `liveletters-sub` — TECHNICAL_SPEC

## 1. Цель

КреЙт реализует команду `lltt sub`. Это **источник истины для подписок текущего пользователя** — он:

- обновляет локальный TOML (`meta.subscriptions`);
- пишет запись в таблицу `subscriptions` (зеркало для быстрых запросов fan-out);
- ставит в `outbox` событие `subscription_changed` (чтобы владелец блога узнал о подписке).

## 2. Архитектура и зависимости

```
apps/lltt
   └─ clap-разбор → liveletters_sub::Args
                       └─ run()
                            ├─ config (load_identity, save_identity)
                            ├─ domain (ResourceAddress::new, Subscription)
                            └─ app_core
                                 ├─ subscribe(service, SubscribeCommand)
                                 │    └─ store (save_subscription, enqueue_outbox_record)
                                 └─ list_subscriptions(ListSubscriptionsQuery)
                                      └─ store (list_subscriptions_for_resource)
```

Зависимости (`Cargo.toml`): `liveletters-app-core`, `liveletters-config`, `liveletters-domain`, `liveletters-output`, `liveletters-store`, `clap` (для derive в `Args`), `thiserror`, `toml`.

## 3. Структуры

```rust
// src/args.rs
pub struct Args {
    pub tokens: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubAction {
    Subscribe { resource_address: String },
    List,
    Rm        { resource_address: String },
}
```

`Args.tokens` заполняется clap-вариантом `Command::Sub(liveletters_sub::Args)` в `apps/lltt`; clap-атрибуты (`trailing_var_arg`, `allow_hyphen_values`, `num_args = 0..`) заданы в `main.rs` бинаря.

## 4. Алгоритм `run`

```
fn run(ctx, args):
    if args.tokens.is_empty() → Err(NoAddressArgument)
    let action = parse_action(args.tokens):
        if first == "list"                       → List
        if first == "rm" && tokens.len() >= 2    → Rm { address = tokens[1] }
        if tokens.len() == 1                     → Subscribe { address = tokens[0] }
        else                                     → UnknownSubcommand

    let identity = config::load_identity(ctx.home, ctx.identity_name)?
    let service = AppCoreService::new(store)
    let delivery = identity.mail.receive.first()
                   ?? identity.mail.publish

    match action:
        Subscribe:
            service.subscribe(SubscribeCommand {
                resource_address: address (parse → ResourceAddress),
                subscriber_account_id: identity.account_id,
                subscriber_delivery_address: delivery,
            })?
            // subscribe() ВНУТРИ уже:
            //   1) save_subscription() → таблица subscriptions
            //   2) enqueue_outbox_record → событие subscription_changed (action=subscribe)
            //   НО: подписка добавляется в meta.subscriptions в IDENTITY через save_identity
            //   отдельным вызовом в run() (см. ниже).

        List:
            let list = service.list_subscriptions(
                ListSubscriptionsQuery {
                    owned_resource_address: &identity.mail.publish,
                    subscribed_addresses: identity.meta.subscriptions.iter().map(String::as_str),
                }
            )?
            print table

        Rm:
            service.unsubscribe(UnsubscribeCommand { resource_address, subscriber_account_id })?;
            // mirror to TOML: meta.subscriptions remove address
            save_identity(...)? // перезапись файла
```

NB: в текущей реализации команда `subscribe` через `app_core` пишет только в `subscriptions` + `outbox`, а в TOML (`meta.subscriptions`) дописывает сам `run` после успешного вызова. Это сознательное разделение: `app_core` ничего не знает про `config`.

## 5. Ошибки

`SubError` собирает `thiserror`-варианты, перечисленные в INTERFACE.md. В `lib.rs::run` ошибка оборачивается в `Box<dyn Error + Send + Sync>` через `From<SubError> for Box<…>` (реализован вручную, без `?`).

## 6. Что НЕ делается

- Не редактируется `meta.subscriptions` через `app_core` (нет цикла `app_core → config`).
- Не вызывается сетевая отправка (это делает `lltt sync push` в фазе отправки `outbox`).
- Не валидируется, что `resource_address` существует в каких-то глобальных реестрах: подписка считается «локальной» и проверяется только на синтаксис (через `ResourceAddress::new`).

## 7. Совместимость

- Сейчас: текущее поведение команды (см. выше).
- Команда `lltt sub list` может стать частью UI ленты без поломок.
- События `subscription_changed`, поставленные в `outbox`, отправятся в push-фазе `lltt sync`; владелец блога через `liveletters-store::list_subscriptions_for_resource` увидит подписчиков и сможет развернуть (fan-out) свои будущие `PostCreated` адресно.

## 8. Файлы

```
modules/liveletters-sub/
├── Cargo.toml
├── INTERFACE.md
├── TECHNICAL_SPEC.md
├── src/
│   ├── lib.rs     (реэкспорт + run-прокси)
│   ├── args.rs    (Args, SubAction, parse_action)
│   ├── error.rs   (SubError, thiserror)
│   └── run.rs     (бизнес-логика команды)
└── tests/
    ├── common/
    │   └── mod.rs (TestHome, sample_identity)
    └── flow.rs    (6 интеграционных тестов)
```
