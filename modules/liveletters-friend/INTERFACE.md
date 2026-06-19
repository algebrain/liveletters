# `liveletters-friend` — INTERFACE

## Назначение

`liveletters-friend` — библиотечный крейт, реализующий команду
`lltt friend`. Команда добавляет указанный адрес в список друзей текущего
пользователя. Друзья получают право видеть записи с видимостью
`friends_only`, но не становятся подписчиками автоматически.

## Где находится интерфейс

- crate: `liveletters-friend`
- точка подключения: [`src/lib.rs`](src/lib.rs)
- разбор аргументов: [`src/args.rs`](src/args.rs)
- ошибки команды: [`src/error.rs`](src/error.rs)
- алгоритм `run`: [`src/run.rs`](src/run.rs)

## Публичный API

```rust
pub use args::Args;
pub use error::FriendError;
pub use liveletters_output::CommandContext;
pub use run::run;

pub const COMMAND_NAME: &str;
pub fn summary() -> &'static str;
pub fn crate_name() -> &'static str;
```

`run` имеет фиксированную сигнатуру:

```rust
pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

## `Args`

```rust
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Адрес ресурса пользователя, которого нужно добавить в друзья.
    pub address: String,
}
```

Поверхность CLI: `lltt friend <адрес>`.

## Поведение

Если текущий пользователь уже подписан на указанный адрес, команда сразу:

- сохраняет пару «мой ресурс → друг» в таблице `friends`;
- кладёт в `outbox` событие `friend_added`.

Если подписки ещё нет, команда:

- создаёт запись в `pending_friends`;
- отправляет обычный `SubscriptionRequested` на адрес друга;
- после получения `SubscriptionConfirmed` прикладный слой завершает
  добавление в друзья и ставит отдельное событие `friend_added`.

Протокол подписки при этом остаётся обычным: в `SubscriptionRequested` нет
дополнительного поля, которое превращало бы его в «запрос дружбы».

## Что печатает

```text
запрошено добавление в друзья: bob@example.org
```

или, если подписка уже была подтверждена:

```text
добавлен в друзья: bob@example.org
```

## Соседи

- [`liveletters-app-core`](../../modules/liveletters-app-core/INTERFACE.md) — `AppCore::friend`.
- [`liveletters-store`](../../modules/liveletters-store/INTERFACE.md) — `friends`, `pending_friends`, `friend_of`.
- [`liveletters-output`](../../modules/liveletters-output/INTERFACE.md) — `CommandContext`.

## Тесты

- `apps/lltt/tests/cli_friend.rs` — команда `lltt friend` через бинарь.
- `modules/liveletters-app-core/tests/friends.rs` — прикладные сценарии
  добавления в друзья.
