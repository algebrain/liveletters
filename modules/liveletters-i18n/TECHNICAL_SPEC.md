# liveletters-i18n

## Назначение

Крейт локализации строк-шаблонов для писем LiveLetters. Предоставляет переводы для двух локалей и подстановку переменных.

## Зона ответственности

- хранение шаблонов строк на русском и английском языках для всех типов событий;
- подстановка переменных через плейсхолдеры `%name%`;
- определение `Locale::Ru` / `Locale::En` и парсинг из строки;
- определение системной локали через `detect_system_locale` (переменные окружения `LC_ALL`/`LC_MESSAGES`/`LANG`).

## Зависимости

- `thiserror` — типизированные ошибки в `I18nError`.

Не зависит от других крейтов LiveLetters.

## Структура

- `src/locale.rs` — `Locale` enum, `parse_locale`, `detect_system_locale`;
- `src/templates.rs` — таблица шаблонов `(key, locale) → &str`;
- `src/translate.rs` — `translate(key, locale, vars)` с подстановкой переменных;
- `src/error.rs` — `I18nError` (UnknownLocale, UnknownKey, MissingVariable);
- `tests/translate.rs` — покрытие всех ключей и переменных.
