use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::config::LogLevel;

enum Sink {
    File(BufWriter<std::fs::File>, PathBuf),
    Stderr,
    None,
}

static WRITER: Mutex<Option<Sink>> = Mutex::new(None);

/// Записать одну строку в текущий открытый файл / stderr.
/// Выполняет ротацию, если текущий размер превышает `max_size`.
pub fn write_message(level: LogLevel, target: &str, message: &str, max_size: u64, keep_files: u32) {
    let now = current_iso8601();
    let level_str = level.as_str();
    let line = format!("{now} {level_str} {target} {message}\n");

    let mut guard = match WRITER.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let Some(sink) = guard.as_mut() else {
        return;
    };

    match sink {
        Sink::None => {}
        Sink::Stderr => {
            let _ = std::io::stderr().write_all(line.as_bytes());
        }
        Sink::File(writer, path) => {
            if let Err(err) = writer.write_all(line.as_bytes()) {
                eprintln!("liveletters-log: ошибка записи: {err}");
                return;
            }
            if let Err(err) = writer.flush() {
                eprintln!("liveletters-log: ошибка сброса буфера: {err}");
                return;
            }
            let needs_rotation = std::fs::metadata(path.as_path())
                .map(|m| m.len() >= max_size)
                .unwrap_or(false);
            if needs_rotation {
                if let Err(err) = writer.flush() {
                    eprintln!("liveletters-log: ошибка сброса перед ротацией: {err}");
                }
                if let Err(err) = crate::rotation::rotate(path.as_path(), keep_files) {
                    eprintln!("liveletters-log: ошибка ротации: {err}");
                    return;
                }
                match open_append(path.as_path()) {
                    Ok(file) => {
                        *writer = BufWriter::new(file);
                    }
                    Err(err) => {
                        eprintln!("liveletters-log: не удалось переоткрыть файл: {err}");
                    }
                }
            }
        }
    }
}

/// Открыть файл на дозапись (создаёт при отсутствии).
pub(crate) fn open_append(path: &Path) -> std::io::Result<std::fs::File> {
    OpenOptions::new().create(true).append(true).open(path)
}

pub(crate) fn install(file: std::fs::File, path: PathBuf) {
    if let Ok(mut g) = WRITER.lock() {
        *g = Some(Sink::File(BufWriter::new(file), path));
    }
}

pub(crate) fn install_stderr() {
    if let Ok(mut g) = WRITER.lock() {
        *g = Some(Sink::Stderr);
    }
}

pub(crate) fn install_none() {
    if let Ok(mut g) = WRITER.lock() {
        *g = Some(Sink::None);
    }
}

pub(crate) fn shutdown_writer() {
    if let Ok(mut g) = WRITER.lock() {
        if let Some(Sink::File(mut w, _)) = g.take() {
            let _ = w.flush();
        } else {
            *g = None;
        }
    }
}

fn current_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();

    let days = (secs / 86_400) as i64;
    let secs_in_day = (secs % 86_400) as u32;
    let hour = secs_in_day / 3600;
    let minute = (secs_in_day % 3600) / 60;
    let second = secs_in_day % 60;

    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

/// Преобразовать число дней от 1970-01-01 в (год, месяц, день).
/// Основано на алгоритме Howard Hinnant.
fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + (if m <= 2 { 1 } else { 0 });
    (y as i32, m as u32, d as u32)
}
