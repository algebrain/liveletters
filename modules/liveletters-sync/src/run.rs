use crate::{Args, SyncError, CommandContext};

/// Заглушка запуска команды `lltt sync`.
///
/// Реальная сетевая синхронизация реализована в крейте `liveletters-lltt-sync`
/// (подкоманды `lltt sync pull` и `lltt sync push`). Этот модуль остался как
/// слой совместимости, к которому обращается подсистема `ingest` через
/// `liveletters-sync::SyncEngine::ingest_batch`; отдельный `run` здесь
/// не вызывается.
pub fn run(_ctx: &CommandContext, _args: &Args) -> Result<(), SyncError> {
    unimplemented!("сетевая синхронизация доступна через `lltt sync pull|push` (крейт liveletters-lltt-sync)")
}
