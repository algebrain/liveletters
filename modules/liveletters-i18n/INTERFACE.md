# `liveletters-i18n`

Крейт содержит локализацию строк-шаблонов для писем LiveLetters.
Предоставляет переводы для двух локалей: `ru` (по умолчанию) и `en`.
Шаблоны подставляют значения в плейсхолдеры вида `%имя%`.

## Публичный API

### `Locale`

```rust
pub enum Locale { Ru, En }
```

Допустимые локали. Других значений нет.

### `parse_locale(value: &str) -> Result<Locale, I18nError>`

Парсит строку из пользовательских настроек в `Locale`.
Принимает строки вроде `"ru"`, `"en"`, обрезанные варианты (`"  EN "` -> `Locale::En`).
Регистр не различается. Любое другое значение -> `I18nError::UnknownLocale`.

### `detect_system_locale() -> Locale`

Определяет локаль по переменным окружения `LC_ALL`, `LC_MESSAGES`, `LANG`
(в этом порядке). Из строки вроде `"ru_RU.UTF-8"` берётся первая часть
до `_`, `-` или `.` и парсится через `parse_locale`.
Если ничего не задано или значение не входит в поддерживаемые локали —
возвращается `Locale::En`. Используется приложением как дефолт при
создании новой записи пользователя.

### `Vars<'a>(pub &'a [(&'a str, &'a str)])`

Набор пар «имя — значение», которые подставляются в шаблон.
Передаётся в `translate` третьим аргументом.

### `translate(key: &str, locale: Locale, vars: Vars) -> Result<String, I18nError>`

Подставляет значения в шаблон по ключу и локали.
Возвращает итоговую строку, либо ошибку:
- `I18nError::UnknownKey` — ключ не описан в таблице шаблонов;
- `I18nError::MissingVariable(name)` — в шаблоне есть `%name%`, но в `vars` его нет.

## Ключи шаблонов

Шаблоны делятся на две части: `subject` и `body`.
Все события имеют обе части.

| Ключ | Поля | Назначение |
|------|------|------------|
| `post_created.subject` | `sender`, `resource` | Тема письма о новой записи. |
| `post_created.body` | `sender`, `resource`, `body` | Тело письма о новой записи. |
| `comment_created.subject` | `sender` | Тема письма о новом комментарии. |
| `comment_created.body` | `sender`, `post_id`, `body` | Тело письма о новом комментарии. |
| `comment_edited.subject` | `sender` | Тема письма об изменении комментария. |
| `comment_edited.body` | `sender`, `post_id`, `body` | Тело письма об изменении комментарии. |
| `post_hidden.subject` | `actor` | Тема письма о скрытии записи. |
| `post_hidden.body` | `actor`, `post_id` | Тело письма о скрытии записи. |
| `comment_created_redistribute.subject` | `resource` | Тема пересылки комментария подписчикам ресурса. |
| `comment_created_redistribute.body` | `sender`, `post_id`, `body` | Тело пересылки комментария подписчикам ресурса. |
| `subscription_requested.subject` | `subscriber` | Тема письма с запросом подписки (B → A). |
| `subscription_requested.body` | `subscriber`, `resource` | Тело письма с запросом подписки (B → A). |
| `subscription_confirmed_accepted.subject` | `owner`, `resource` | Тема письма о подтверждении подписки (A → B). |
| `subscription_confirmed_accepted.body` | `owner`, `resource` | Тело письма о подтверждении подписки (A → B). |
| `subscription_confirmed_declined.subject` | `owner` | Тема письма об отказе в подписке (A → B). |
| `subscription_confirmed_declined.body` | `owner`, `resource` | Тело письма об отказе в подписке (A → B). |
| `subscription_revoked.subject` | `subscriber` | Тема письма об отписке (B → A). |
| `subscription_revoked.body` | `subscriber`, `resource` | Тело письма об отписке (B → A). |

Символы, не входящие в `%...%`, передаются в результат без изменений.

## Использование

```rust
use liveletters_i18n::{Locale, Vars, parse_locale, translate};

let locale = parse_locale("ru").unwrap();
let vars = Vars(&[("sender", "Алиса"), ("resource", "blog-1")]);
let subject = translate("post_created.subject", locale, vars).unwrap();
```

## Локализация только отправителя

Локализация применяется к письмам, которые генерирует **отправитель**
(локальный пользователь). Получатель читает письмо на языке отправителя —
это сознательное упрощение, принятое для блоговой платформы.
