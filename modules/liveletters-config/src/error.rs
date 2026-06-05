#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Io(String),
    Toml(String),
    MissingField { field: &'static str },
    UnknownIdentity(String),
    NoCurrentUser(std::path::PathBuf),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "io error: {message}"),
            Self::Toml(message) => write!(f, "toml error: {message}"),
            Self::MissingField { field } => write!(f, "missing required field: {field}"),
            Self::UnknownIdentity(name) => write!(f, "unknown identity: {name}"),
            Self::NoCurrentUser(path) => {
                write!(
                    f,
                    "текущий пользователь liveletters не задан (файл `{}` отсутствует); создайте пользователя командой `lltt user init <имя>`, добавьте его командой `lltt user add <имя> --from <файл>`, затем выберите командой `lltt cu <имя>`",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        Self::Toml(error.to_string())
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Toml(error.to_string())
    }
}
