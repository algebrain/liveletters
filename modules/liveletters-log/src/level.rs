use std::sync::atomic::{AtomicU8, Ordering};

use crate::config::LogLevel;

/// Текущий активный уровень. Хранится в `AtomicU8` для дешёвого fast-path в макросах.
static CURRENT_LEVEL: AtomicU8 = AtomicU8::new(0);

pub fn set_level(level: LogLevel) {
    CURRENT_LEVEL.store(level.as_u8(), Ordering::Release);
}

pub fn current_level() -> LogLevel {
    match CURRENT_LEVEL.load(Ordering::Acquire) {
        1 => LogLevel::Error,
        2 => LogLevel::Warn,
        3 => LogLevel::Info,
        4 => LogLevel::Debug,
        5 => LogLevel::Trace,
        _ => LogLevel::Off,
    }
}

pub fn is_enabled(level: LogLevel) -> bool {
    level.as_u8() != 0 && level.as_u8() <= CURRENT_LEVEL.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_ordering_matches_severity() {
        assert!(LogLevel::Error.as_u8() < LogLevel::Warn.as_u8());
        assert!(LogLevel::Warn.as_u8() < LogLevel::Info.as_u8());
        assert!(LogLevel::Info.as_u8() < LogLevel::Debug.as_u8());
        assert!(LogLevel::Debug.as_u8() < LogLevel::Trace.as_u8());
        assert_eq!(LogLevel::Off.as_u8(), 0);
    }

    #[test]
    fn is_enabled_respects_global_level() {
        set_level(LogLevel::Off);
        assert!(!is_enabled(LogLevel::Error));
        set_level(LogLevel::Info);
        assert!(is_enabled(LogLevel::Error));
        assert!(is_enabled(LogLevel::Info));
        assert!(!is_enabled(LogLevel::Debug));
    }
}
