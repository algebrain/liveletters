use std::path::Path;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};

use crate::config::{LogConfig, LogDestination};
use crate::level;
use crate::writer;

static ACTIVE_MAX_SIZE: AtomicU64 = AtomicU64::new(DEFAULT_MAX_SIZE);
static ACTIVE_KEEP: AtomicU32 = AtomicU32::new(DEFAULT_KEEP);
static ACTIVE_INCLUDE_BODIES: AtomicU8 = AtomicU8::new(0);
static ACTIVE_DESTINATION: AtomicU8 = AtomicU8::new(0);

const MIN_MAX_SIZE: u64 = 1024;
const DEFAULT_MAX_SIZE: u64 = 5 * 1024 * 1024;
const DEFAULT_KEEP: u32 = 3;

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("ошибка ввода-вывода при инициализации журнала: {0}")]
    Io(#[from] std::io::Error),
}

/// Инициализирует глобальный логгер для процесса.
///
/// - `home` — путь к `${LIVELETTERS_HOME}`.
/// - `config` — параметры из `GlobalConfig.log`.
///
/// Создаёт каталог `${home}/logs/`, открывает файл
/// `${home}/logs/liveletters.log` (если `destination = File`),
/// выставляет глобальный уровень.
///
/// Повторный вызов в том же процессе — `Ok(())` без побочных эффектов.
pub fn init(home: &Path, config: &LogConfig) -> Result<(), LogError> {
    let destination = config.destination;
    let level_value = config.level;
    let max_size = clamp_max_size(config.max_size_bytes);
    let keep = clamp_keep_files(config.keep_files);

    if max_size != config.max_size_bytes {
        eprintln!(
            "liveletters-log: max_size_bytes={} ниже минимума, использую {max_size}",
            config.max_size_bytes
        );
    }
    if keep != config.keep_files {
        eprintln!(
            "liveletters-log: keep_files={} ниже минимума, использую {keep}",
            config.keep_files
        );
    }

    match destination {
        LogDestination::File => {
            let dir = home.join("logs");
            std::fs::create_dir_all(&dir)?;
            let path = dir.join("liveletters.log");
            let file = writer::open_append(&path)?;
            writer::install(file, path);
        }
        LogDestination::Stderr => {
            writer::install_stderr();
        }
        LogDestination::None => {
            writer::install_none();
        }
    }
    ACTIVE_DESTINATION.store(destination as u8, Ordering::Release);

    level::set_level(level_value);
    ACTIVE_MAX_SIZE.store(max_size, Ordering::Release);
    ACTIVE_KEEP.store(keep, Ordering::Release);
    ACTIVE_INCLUDE_BODIES.store(u8::from(config.include_bodies), Ordering::Release);
    Ok(())
}

/// Сбрасывает буфер writer'а. Вызывайте перед корректным завершением.
pub fn shutdown() {
    writer::shutdown_writer();
}

pub fn is_bodies_enabled() -> bool {
    ACTIVE_INCLUDE_BODIES.load(Ordering::Acquire) != 0
}

pub fn max_size() -> u64 {
    ACTIVE_MAX_SIZE.load(Ordering::Acquire)
}

pub fn keep_files() -> u32 {
    ACTIVE_KEEP.load(Ordering::Acquire)
}

pub fn reset_for_tests() {
    level::set_level(crate::config::LogLevel::Off);
    ACTIVE_MAX_SIZE.store(DEFAULT_MAX_SIZE, Ordering::Release);
    ACTIVE_KEEP.store(DEFAULT_KEEP, Ordering::Release);
    ACTIVE_INCLUDE_BODIES.store(0, Ordering::Release);
    ACTIVE_DESTINATION.store(0, Ordering::Release);
    writer::shutdown_writer();
}

fn clamp_max_size(value: u64) -> u64 {
    if value == 0 {
        DEFAULT_MAX_SIZE
    } else {
        value.max(MIN_MAX_SIZE)
    }
}

fn clamp_keep_files(value: u32) -> u32 {
    if value == 0 { DEFAULT_KEEP } else { value }
}
