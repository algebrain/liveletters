# `liveletters-utils`: общие утилиты

## Назначение

`liveletters-utils` содержит небольшие функции и типы, которые нужны в разных частях проекта и не относятся к хранению, почтовому транспорту, протоколу или командам `lltt`.

Крейт нужен для единообразного поведения: одна проверка почтового адреса, один разбор строки протокольной идентичности, одна обработка пустых строк и времени.

## Публичные модули

### `email`

```rust
pub fn looks_like_email(value: &str) -> bool
pub fn email_local_part(value: &str) -> Option<&str>
```

`looks_like_email` проверяет простую форму адреса: одна `@`, непустая локальная часть, непустой домен, без пробелов внутри.

`email_local_part` возвращает часть адреса до `@`, если адрес проходит ту же простую проверку.

Это не полный разбор почтовых адресов по стандартам. Функции предназначены для уже существующих сценариев LiveLetters, где нужен предсказуемый и узкий контракт.

### `protocol_identity`

```rust
pub struct ProtocolIdentity;

impl ProtocolIdentity {
    pub fn new(nickname: impl Into<String>, email: impl Into<String>) -> Result<Self, ProtocolIdentityError>;
    pub fn parse(input: &str) -> Result<Self, ProtocolIdentityError>;
    pub fn nickname(&self) -> &str;
    pub fn email(&self) -> &str;
    pub fn to_wire_string(&self) -> String;
}
```

`ProtocolIdentity` описывает строковую идентичность протокола вида:

```text
Alice <alice@example.com>
```

Именно в таком виде значения сериализуются в JSON-поля `origin` и `source`. При разборе внешние пробелы вокруг имени и адреса обрезаются, а канонический вывод всегда имеет один пробел перед `<`.

Ошибки разбора представлены типом `ProtocolIdentityError`:

- `InvalidWireFormat` — строка не имеет формы `Имя <адрес>`;
- `BlankNickname` — имя пустое;
- `BlankEmail` — адрес пустой;
- `InvalidEmail` — адрес не проходит простую проверку формы.

Тип реализует `Serialize`, `Deserialize` и `Display`.

### `text`

```rust
pub struct NonBlankError {
    pub field: &'static str,
}

pub fn require_non_blank<'a>(value: &'a str, field: &'static str) -> Result<&'a str, NonBlankError>
```

`require_non_blank` проверяет, что строка не пустая и не состоит только из пробелов. Функция возвращает исходную строку без обрезания: вызывающий сам решает, являются ли пробелы значимыми.

### `time`

```rust
pub fn unix_now() -> u64
pub fn unix_secs(time: SystemTime) -> u64
```

`unix_secs` переводит `SystemTime` в секунды Unix. Время до эпохи Unix считается нулём.

`unix_now` возвращает текущее время в секундах Unix.

## Что крейт не делает

- не читает и не пишет файлы;
- не обращается к базе;
- не отправляет и не получает почту;
- не знает о командах `lltt`;
- не выполняет полный разбор почтовых адресов по стандартам.

