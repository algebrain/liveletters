use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Выполняет ротацию безусловно (без проверки размера).
/// Вызывается из `writer` после того, как `metadata` подтвердил превышение лимита.
pub fn rotate(log_path: &Path, keep_files: u32) -> io::Result<()> {
    if keep_files == 0 {
        return Ok(());
    }

    // Самый старый файл удаляем молча.
    let oldest = with_index(log_path, keep_files);
    let _ = fs::remove_file(&oldest);

    // Сдвигаем .(N-1) -> .N, ..., .1 -> .2
    if keep_files > 1 {
        for i in (1..keep_files).rev() {
            let from = with_index(log_path, i);
            let to = with_index(log_path, i + 1);
            if from.exists() {
                fs::rename(&from, &to)?;
            }
        }
    }

    // Текущий -> .1
    fs::rename(log_path, with_index(log_path, 1))?;
    Ok(())
}

/// Сдвигает старые файлы журнала и переименовывает текущий в `.1`.
///
/// Возвращает `true`, если ротация была выполнена.
pub fn rotate_if_needed(log_path: &Path, max_size_bytes: u64, keep_files: u32) -> io::Result<bool> {
    let meta = match fs::metadata(log_path) {
        Ok(m) => m,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    if meta.len() < max_size_bytes {
        return Ok(false);
    }
    rotate(log_path, keep_files)?;
    Ok(true)
}

fn with_index(log_path: &Path, index: u32) -> PathBuf {
    let parent = log_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = log_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("liveletters");
    parent.join(format!("{stem}.log.{index}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_bytes(path: &Path, bytes: usize) {
        let mut f = fs::File::create(path).unwrap();
        let chunk: [u8; 64] = [b'x'; 64];
        let mut written = 0_usize;
        while written < bytes {
            let to_write = (bytes - written).min(chunk.len());
            f.write_all(&chunk[..to_write]).unwrap();
            written += to_write;
        }
        f.sync_all().unwrap();
    }

    #[test]
    fn rotation_moves_current_to_dot1() {
        let tmp = tempfile::tempdir().unwrap();
        let log = tmp.path().join("liveletters.log");
        write_bytes(&log, 2048);
        let original_bytes = fs::read(&log).unwrap().len() as u64;

        let rotated = rotate_if_needed(&log, 1024, 3).unwrap();
        assert!(rotated);
        assert!(!log.exists());
        let rotated_one = tmp.path().join("liveletters.log.1");
        assert!(rotated_one.exists());
        assert_eq!(fs::metadata(&rotated_one).unwrap().len(), original_bytes);
    }

    #[test]
    fn rotation_drops_files_above_keep_count() {
        let tmp = tempfile::tempdir().unwrap();
        let log = tmp.path().join("liveletters.log");
        // Предзаполним архив: .1, .2, .3 (последний должен быть удалён).
        for i in 1..=3 {
            let p = tmp.path().join(format!("liveletters.log.{i}"));
            write_bytes(&p, 256);
        }
        write_bytes(&log, 2048);

        rotate_if_needed(&log, 1024, 3).unwrap();

        assert!(!tmp.path().join("liveletters.log.4").exists());
        assert!(tmp.path().join("liveletters.log.1").exists());
        assert!(tmp.path().join("liveletters.log.2").exists());
        assert!(tmp.path().join("liveletters.log.3").exists());
    }

    #[test]
    fn no_rotation_when_below_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let log = tmp.path().join("liveletters.log");
        write_bytes(&log, 100);

        let rotated = rotate_if_needed(&log, 1024, 3).unwrap();
        assert!(!rotated);
        assert!(log.exists());
    }

    #[test]
    fn no_rotation_when_file_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let log = tmp.path().join("liveletters.log");
        let rotated = rotate_if_needed(&log, 1024, 3).unwrap();
        assert!(!rotated);
    }
}
