use crate::CuError;

pub fn validate_user_name(name: &str) -> Result<(), CuError> {
    if name.trim().is_empty() {
        return Err(CuError::InvalidArgs(
            "имя пользователя не может быть пустым".to_owned(),
        ));
    }
    if name == "." || name == ".." {
        return Err(CuError::InvalidArgs(format!(
            "недопустимое имя пользователя: {name}"
        )));
    }
    if name
        .chars()
        .any(|ch| ch.is_whitespace() || ch == '/' || ch == '\\')
    {
        return Err(CuError::InvalidArgs(format!(
            "недопустимое имя пользователя: {name}"
        )));
    }
    Ok(())
}
