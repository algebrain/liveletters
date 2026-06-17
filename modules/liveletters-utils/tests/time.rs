use std::time::{Duration, SystemTime, UNIX_EPOCH};

use liveletters_utils::time::unix_secs;

#[test]
fn unix_epoch_is_zero_seconds() {
    assert_eq!(unix_secs(UNIX_EPOCH), 0);
}

#[test]
fn time_after_epoch_is_whole_seconds() {
    let time = UNIX_EPOCH + Duration::from_secs(1_710_000_123);

    assert_eq!(unix_secs(time), 1_710_000_123);
}

#[test]
fn time_before_epoch_is_clamped_to_zero() {
    let time = UNIX_EPOCH - Duration::from_secs(10);

    assert_eq!(unix_secs(time), 0);
}

#[test]
fn current_time_is_not_before_epoch() {
    assert!(unix_secs(SystemTime::now()) > 0);
}
