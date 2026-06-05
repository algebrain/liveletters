//! Утилиты форматирования времени в UTC.
//!
//! Поддерживает диапазон 1970–2100. Для меток времени за пределами
//! 2100 (`unix_secs > 4_102_444_800`, т.е. `2100-01-01 00:00:00 UTC`)
//! функция возвращает `2100-01-01 00:00:00 UTC` — это сознательное
//! упрощение, не ошибка. Точность — до секунд; в будущем, если
//! потребуется работа с датами в `liveletters-diagnostics`,
//! планируется переход на `chrono`.

/// Преобразует Unix-время (секунды с эпохи) в `(год, месяц, день, час, минута, секунда)` UTC.
/// Поддерживает годы 1970–2100; високосные годы считаются по правилу
/// «год делится на 4, кроме кратных 100, но не 400».
pub fn unix_to_ymdhms(unix_secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let days = unix_secs / 86_400;
    let secs_of_day = unix_secs % 86_400;
    let hour = (secs_of_day / 3600) as u32;
    let minute = ((secs_of_day % 3600) / 60) as u32;
    let second = (secs_of_day % 60) as u32;

    let (year, month, day) = days_to_ymd(days);
    (year, month, day, hour, minute, second)
}

/// Форматирует Unix-время в строку вида `1970-01-01 00:00:00 UTC`.
pub fn format_unix_iso8601_utc(unix_secs: u64) -> String {
    let (year, month, day, hour, minute, second) = unix_to_ymdhms(unix_secs);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

fn days_to_ymd(days_since_epoch: u64) -> (u32, u32, u32) {
    let mut year = 1970u32;
    let mut remaining = days_since_epoch;

    loop {
        let leap = is_leap(year);
        let year_days = if leap { 366 } else { 365 };
        if remaining < year_days {
            break;
        }
        remaining -= year_days;
        year += 1;
        if year > 2100 {
            return (year, 1, 1);
        }
    }

    let leap = is_leap(year);
    let months = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &m_days in &months {
        if remaining < m_days {
            return (year, month, remaining as u32 + 1);
        }
        remaining -= m_days;
        month += 1;
    }
    (year, month, 1)
}

fn is_leap(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970_01_01() {
        let (y, m, d, h, mi, s) = unix_to_ymdhms(0);
        assert_eq!((y, m, d, h, mi, s), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn handles_leap_year() {
        let (y, m, d, _, _, _) = unix_to_ymdhms(951_782_400);
        assert_eq!((y, m, d), (2000, 2, 29));
    }

    #[test]
    fn formats_iso8601() {
        assert_eq!(format_unix_iso8601_utc(0), "1970-01-01 00:00:00 UTC");
    }

    #[test]
    fn clamps_dates_beyond_2100() {
        let far_future = 4_102_444_800;
        assert_eq!(
            format_unix_iso8601_utc(far_future),
            "2100-01-01 00:00:00 UTC"
        );
    }
}
