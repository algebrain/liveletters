# `liveletters-secret-box` INTERFACE

## Назначение

`liveletters-secret-box` это узкоспециализированная криптографическая обёртка для защиты секретов, которые нельзя оставлять в БД в открытом виде, но которые при этом не требуют полноценной KMS-инфраструктуры.

Типичный и единственный сценарий использования — пароль IMAP/SMTP-учётки почтового транспорта, который должен:

- лежать в SQLite, чтобы пользователю не приходилось вводить его каждый раз;
- не лежать в SQLite в виде открытого текста, иначе любое копирование файла БД = утечка пароля;
- расшифровываться ровно тем процессом, который запущен от имени того же пользователя.

Для этого крейт предоставляет один конкретный механизм:

- шифр XChaCha20-Poly1305 с 32-байтным ключом, лежащим отдельным файлом;
- текстовый токен формата `obf:v1:<base64>`, пригодный для хранения в текстовом столбце БД;
- отдельный API `open`/`open_or_create` для разделения сценариев «ключ уже есть» и «первый запуск».

Крейт не пытается быть:

- менеджером паролей;
- KMS-клиентом;
- менеджером учётных данных операционной системы;
- обобщённым «secure storage».

Это одна конкретная утилита для одного конкретного сценария: превратить пароль почты в `obf:v1:…` и обратно, опираясь на локальный файл ключа с правами 0o600.

## Где находится интерфейс

- crate: `liveletters-secret-box`
- точка подключения: `src/lib.rs`

Наружу экспортируются:

- структура `SecretBox`;
- тип ошибки `SecretBoxError`;
- функция-хелпер `default_key_path(data_dir)`.

Внутренние модули `codec`, `error`, `key_file` не публикуются.

## Что считается внешним интерфейсом этого модуля

С практической точки зрения внешний интерфейс `liveletters-secret-box` это:

1. типы `SecretBox` и `SecretBoxError`;
2. способы получить `SecretBox`: `open` (строгий) и `open_or_create` (создаёт ключ при первом запуске);
3. методы `obfuscate` / `deobfuscate` для превращения секрета в токен и обратно;
4. предикат `is_obfuscated` для отличения обфусцированного значения от открытого;
5. функция `default_key_path(data_dir)` для расчёта канонического пути к файлу ключа.

Именно этим API пользуется `liveletters-store` (через свой `secret_bridge.rs`) и, косвенно, любой код, который хочет класть секрет в `mail_settings` без открытого текста.

## Главный объект: `SecretBox`

### Зачем нужен

`SecretBox` это не «общий сейф для всего», а узкоспециализированный шифр-объект с тремя свойствами:

- привязан к конкретному пути файла ключа на диске;
- хранит сам ключ в памяти в виде `[u8; 32]`, чтобы не дёргать файл при каждом `obfuscate`/`deobfuscate`;
- предоставляет API, который симметричен относительно `obfuscate` / `deobfuscate` (что зашифровали — то и расшифровали), но при этом не пытается прятать `obfuscate`/`deobfuscate` за trait-объектом: это конкретный XChaCha20-Poly1305, а не семейство алгоритмов.

Этого достаточно, чтобы:

- верхние слои могли писать в БД безопасный токен вместо пароля;
- верхние слои не работали с `chacha20poly1305`, `base64` и путями к файлу ключа напрямую;
- тесты могли создавать «свежий» сейф в `tempfile::TempDir` без сложной инфраструктуры.

### Конструкторы: `open` vs `open_or_create`

В крейте сознательно два разных конструктора, и они **не взаимозаменяемы**.

#### `SecretBox::open(key_path)`

```rust
pub fn open(key_path: &Path) -> Result<Self, SecretBoxError>
```

Строгий конструктор:

- читает файл ключа;
- проверяет, что длина ровно 32 байта;
- возвращает `SecretBoxError::Io`, если файла нет;
- возвращает `SecretBoxError::InvalidKeyLength { path, expected, actual }`, если файл есть, но длина неверная.

Используется в сценариях, где ключ должен уже существовать к этому моменту: например, обычный запуск приложения, у которого `mail-password-obfuscation.key` лежит с прошлой сессии.

#### `SecretBox::open_or_create(key_path)`

```rust
pub fn open_or_create(key_path: &Path) -> Result<Self, SecretBoxError>
```

Создающий конструктор:

- если файл ключа существует — ведёт себя как `open`;
- если файла нет — генерирует 32 случайных байта через `OsRng`, создаёт родительский каталог при необходимости, записывает файл;
- на Unix выставляет права 0o600 (через `std::os::unix::fs::PermissionsExt`);
- возвращает `SecretBox` с только что записанным ключом.

Используется в сценариях first-run, в тестах, а также в `liveletters-store::secret_bridge::load_or_create`.

#### Почему не один конструктор

Разделение ответственности явное:

- `open` — «я ожидаю, что ключ уже есть, и я не имею права его генерировать»;
- `open_or_create` — «если ключа нет, сгенерируй; иначе прочитай».

Это удерживает caller от случайной перезаписи существующего ключа новым случайным значением, что было бы катастрофой: после такой перезаписи все ранее сохранённые обфусцированные токены стали бы нерасшифровываемыми.

### `key_path(&self) -> &Path`

Возвращает путь к файлу ключа, с которым открыт текущий сейф. Полезен для диагностики и логирования. Сам ключ наружу не отдаётся.

### `obfuscate(&self, plaintext: &str) -> Result<String, SecretBoxError>`

Превращает открытый секрет в токен формата `obf:v1:<base64(nonce || ciphertext)>`:

- `nonce` — 24 случайных байта, уникальных для каждого вызова (XChaCha20 использует 192-битный nonce, что делает случайный выбор безопасным);
- `ciphertext` — XChaCha20-Poly1305(`plaintext`);
- результат кодируется в base64 (стандартный алфавит с паддингом `=`).

Гарантии:

- для одного и того же `plaintext` и одного и того же ключа токены **различаются** (из-за случайного nonce);
- токен начинается с префикса `obf:v1:` — это одновременно и маркер версии формата, и быстрый способ отличить обфусцированное значение от открытого текста;
- внутри `obf:v1:` нет ни пользовательских данных, ни идентификаторов — это чистый криптографический токен.

### `deobfuscate(&self, stored: &str) -> Result<String, SecretBoxError>`

Обратная операция:

- проверяет, что `stored` начинается с `obf:v1:` (иначе `SecretBoxError::InvalidFormat`);
- декодирует base64 (иначе `SecretBoxError::InvalidFormat`);
- проверяет, что после декодирования длина строго больше 24 байт (иначе `SecretBoxError::InvalidFormat` — слишком короткий payload, нет места под `nonce`+`Poly1305`-tag);
- разделяет `nonce` и `ciphertext`, расшифровывает XChaCha20-Poly1305;
- при несовпадении Poly1305-tag возвращает `SecretBoxError::Crypto` — это **намеренно неотличимо от ошибки «неверный ключ»**;
- проверяет, что результат — валидный UTF-8 (иначе `SecretBoxError::InvalidFormat`).

`SecretBoxError::Crypto` никогда не раскрывает, в чём именно проблема: неправильный ключ, испорченный токен или подделанный payload для внешнего наблюдателя выглядят одинаково. Это сознательный выбор, чтобы по сообщению об ошибке нельзя было делать оракул-атаки.

### `is_obfuscated(stored: &str) -> bool`

Чистый предикат без побочных эффектов:

- возвращает `true`, если `stored` начинается с `obf:v1:`;
- возвращает `false` для пустой строки и для открытого текста.

Используется верхними слоями, чтобы решить: токен перед нами, который надо расшифровывать, или открытое значение, которое надо шифровать. Это особенно важно в `liveletters-store::settings::reveal_secret_with_lazy_migration`, где исторически в БД могли лежать оба формата.

### Хелпер: `default_key_path(data_dir)`

```rust
pub fn default_key_path(data_dir: &Path) -> PathBuf
```

Возвращает канонический путь к файлу ключа внутри `data_dir`:

- `data_dir.join("mail-password-obfuscation.key")`.

Используется в `liveletters-store::StorePaths::password_obfuscation_key_path` и в тестах, чтобы все участники кода и тестов сходились в одном имени файла.

## Хранение ключа: `mail-password-obfuscation.key`

Ключ — это 32 случайных байта, сгенерированных через `OsRng`. Файл ключа живёт по пути, который возвращает `default_key_path(data_dir)`.

Свойства файла:

- ровно 32 байта;
- создаётся при первом вызове `open_or_create`;
- на Unix права доступа 0o600;
- при изменении длины файла любое последующее `open` или `open_or_create` возвращает `SecretBoxError::InvalidKeyLength { path, expected, actual }`;
- хранится отдельно от SQLite-файла, поэтому одна только утечка БД не даёт злоумышленнику ключа.

Сценарии отказа ключевого файла:

- файл пропал → `SecretBoxError::Io { source, message: "cannot read key file …" }`;
- файл есть, но длина не 32 байта → `SecretBoxError::InvalidKeyLength`;
- файл есть, но недоступен по правам → `SecretBoxError::Io { source, message: "cannot read key file …" }` или `… cannot set 0o600 on key file …` при первой записи.

Во всех случаях `SecretBoxError` пробрасывается наверх; в `liveletters-store` он транслируется в `StoreError::ProtectedSecretUnavailable` или `StoreError::InvalidProtectedSecretFormat` через `secret_bridge.rs`.

## Ошибки: `SecretBoxError`

```rust
pub enum SecretBoxError {
    Io { source: std::io::Error, message: String },
    InvalidKeyLength { path: PathBuf, expected: usize, actual: usize },
    InvalidFormat { message: String },
    Crypto { message: String },
}
```

Смысл вариантов:

- `Io` — проблема чтения/записи файла ключа или родительского каталога. Поле `source` отдаёт исходную `std::io::Error`, `message` — человекочитаемое описание с указанием пути.
- `InvalidKeyLength` — файл ключа существует, но его длина не равна 32 байтам. Это защита от случайной подмены ключа чем-то посторонним.
- `InvalidFormat` — токен не парсится: нет префикса, не валидный base64, слишком короткий payload, не UTF-8. Используется только в `deobfuscate` и не зависит от ключа.
- `Crypto` — ошибка AEAD: неверный ключ, испорченный ciphertext, подделанный Poly1305-tag. Используется и в `obfuscate` (теоретически, при сбое AEAD), и в `deobfuscate`.

`SecretBoxError` имплементирует `From<std::io::Error>`, поэтому `?` в вызывающем коде пробрасывает IO-ошибки без обёртки. Конструктор `SecretBoxError::io(error, message)` — рекомендуемый путь, когда нужно сохранить и `source`, и человекочитаемое описание.

## Примеры использования

### Создать сейф и обфусцировать пароль

```rust
use liveletters_secret_box::{SecretBox, default_key_path};

let data_dir = std::path::PathBuf::from("/var/lib/liveletters");
let key_path = default_key_path(&data_dir);
let box_ = SecretBox::open_or_create(&key_path)?;

let token = box_.obfuscate("p@ssw0rd-from-mail-config")?;
// token имеет вид "obf:v1:AbCd…==" и безопасно кладётся в mail_settings.password
```

### Прочитать ранее сохранённый токен

```rust
use liveletters_secret_box::{SecretBox, default_key_path};

let key_path = default_key_path(&data_dir);
let box_ = SecretBox::open(&key_path)?; // строго: ключ должен уже быть

let stored = std::env::var("MAIL_PASSWORD")?; // значение из БД
if !SecretBox::is_obfuscated(&stored) {
    return Err(/* legacy plaintext */);
}
let plaintext = box_.deobfuscate(&stored)?;
```

### Тест: round-trip и разные nonce

```rust
#[test]
fn obfuscation_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let box_ = SecretBox::open_or_create(&tmp.path().join("k.bin")).unwrap();
    let token = box_.obfuscate("hunter2").unwrap();
    assert_ne!(token, "hunter2");
    assert!(SecretBox::is_obfuscated(&token));
    assert_eq!(box_.deobfuscate(&token).unwrap(), "hunter2");
}
```

## Что модуль не делает

- не управляет учётными данными ОС (Keychain, Credential Manager, libsecret);
- не реализует ротацию ключей: при потере `mail-password-obfuscation.key` все ранее сохранённые токены нерасшифровываемы;
- не пытается прятать факт наличия зашифрованных данных;
- не защищает ключ от атакующего, который уже работает от имени того же пользователя (например, через keylogger или дамп памяти процесса);
- не предоставляет API для работы с бинарными секретами — только `&str`/`String`, причём проверяет, что после расшифровки получился валидный UTF-8.

## Граница с `liveletters-store`

`liveletters-secret-box` ничего не знает про SQLite, схемы и бизнес-логику. Всё, что он умеет — превращать строку в `obf:v1:…` и обратно по файлу ключа.

`liveletters-store` (через `src/secret_bridge.rs`) поднимает на этом уровень выше:

- знает, где в `data_dir` лежит ключ (`StorePaths::password_obfuscation_key_path`);
- знает, что открытый пароль из `mail_settings.password` надо обфусцировать при сохранении (`obfuscate_secret_if_needed`);
- знает, что при чтении старых plaintext-значений их надо обфусцировать «лениво», не теряя данные (`reveal_secret_with_lazy_migration`);
- транслирует `SecretBoxError` в `StoreError::ProtectedSecretUnavailable` или `StoreError::InvalidProtectedSecretFormat`.

Эта граница сознательно оставлена односторонней: `liveletters-store` зависит от `liveletters-secret-box`, но не наоборот.
