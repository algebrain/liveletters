use std::io::{self, Read};
use std::path::Path;

use liveletters_domain::Visibility;

/// Читает тело команды: из указанного файла или из stdin, если файл не задан.
///
/// Возвращает `Err(String)` с сообщением об ошибке, безопасным для пользователя.
pub fn read_body<R: Read>(body_file: Option<&Path>, stdin: &mut R) -> Result<String, String> {
    match body_file {
        Some(path) => {
            if !path.exists() {
                return Err(format!("файл с телом не найден: {}", path.display()));
            }
            std::fs::read_to_string(path).map_err(|e| format!("ошибка чтения файла: {e}"))
        }
        None => {
            let mut buf = String::new();
            stdin
                .read_to_string(&mut buf)
                .map_err(|e| format!("ошибка чтения stdin: {e}"))?;
            Ok(buf)
        }
    }
}

/// Разбирает строковое значение `--visibility` в `Visibility`.
///
/// Допустимы только `public` и `friends_only`; остальные уровни
/// (`members_only`, `private_community`) добавляются отдельной задачей.
pub fn parse_visibility(raw: &str) -> Result<Visibility, String> {
    match raw {
        "public" => Ok(Visibility::Public),
        "friends_only" => Ok(Visibility::FriendsOnly),
        other => Err(format!(
            "неизвестный уровень видимости: {other} (допустимы: public, friends_only)"
        )),
    }
}

/// Маркер, что тело команды пустое (используется в сообщениях об ошибках).
pub fn body_was_empty() -> String {
    "тело команды пустое".to_owned()
}

/// Обёртка над `read_body` для интеграционных тестов: использует stdin и возвращает
/// `io::Error` напрямую.
pub fn read_body_io<R: Read>(body_file: Option<&Path>, stdin: &mut R) -> io::Result<String> {
    match body_file {
        Some(path) => std::fs::read_to_string(path),
        None => {
            let mut buf = String::new();
            stdin.read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_visibility_accepts_public() {
        assert_eq!(parse_visibility("public").unwrap(), Visibility::Public);
    }

    #[test]
    fn parse_visibility_accepts_friends_only() {
        assert_eq!(
            parse_visibility("friends_only").unwrap(),
            Visibility::FriendsOnly
        );
    }

    #[test]
    fn parse_visibility_rejects_unknown() {
        let err = parse_visibility("members_only").unwrap_err();
        assert!(err.contains("members_only"));
    }

    #[test]
    fn read_body_from_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("body.txt");
        std::fs::write(&path, "Текст из файла").unwrap();

        let mut empty_stdin: &[u8] = b"";
        let body = read_body(Some(&path), &mut empty_stdin).unwrap();
        assert_eq!(body, "Текст из файла");
    }

    #[test]
    fn read_body_from_stdin_when_no_file() {
        let stdin_bytes: &[u8] = "Текст из stdin\n".as_bytes();
        let mut cursor = stdin_bytes;
        let body = read_body(None, &mut cursor).unwrap();
        assert_eq!(body, "Текст из stdin\n");
    }

    #[test]
    fn read_body_missing_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("missing.txt");

        let mut empty_stdin: &[u8] = b"";
        let err = read_body(Some(&path), &mut empty_stdin).unwrap_err();
        assert!(err.contains("не найден"));
    }
}
