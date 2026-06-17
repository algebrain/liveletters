use std::time::{SystemTime, UNIX_EPOCH};

/// Текущее время в секундах Unix.
pub fn unix_now() -> u64 {
    unix_secs(SystemTime::now())
}

/// Время в секундах Unix. Значения до эпохи Unix считаются нулем.
pub fn unix_secs(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
