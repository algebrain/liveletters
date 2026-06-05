#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("настройки почты для {0} отсутствуют; запустите `lltt settings set smtp.host …`")]
    MailSettingsMissing(String),
    #[cfg(feature = "network")]
    #[error("ошибка IMAP: {0}")]
    Imap(String),
    #[cfg(feature = "network")]
    #[error("ошибка SMTP: {0}")]
    Smtp(String),
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),
    #[error("ошибка SyncEngine: {0}")]
    Engine(#[from] liveletters_sync::SyncError),
    #[error("ошибка протокола: {0}")]
    Protocol(String),
    #[error("не удалось декодировать message_body для outbox-записи {0}")]
    OutboxDecode(String),
    #[error("неизвестный режим безопасности почты: {0}")]
    UnknownMailSecurity(String),
}

impl From<liveletters_protocol::ProtocolError> for SyncError {
    fn from(error: liveletters_protocol::ProtocolError) -> Self {
        Self::Protocol(format!("{error:?}"))
    }
}
