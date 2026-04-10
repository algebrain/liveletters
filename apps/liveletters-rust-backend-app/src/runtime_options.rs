use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOptions {
    home_dir: Option<PathBuf>,
}

impl RuntimeOptions {
    pub fn from_args(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut home_dir = None;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            if let Some(value) = arg.to_str().and_then(|text| text.strip_prefix("--home-dir=")) {
                if value.is_empty() {
                    return Err("`--home-dir` requires a non-empty path".into());
                }
                home_dir = Some(PathBuf::from(value));
                continue;
            }

            if arg == "--home-dir" {
                let Some(value) = args.next() else {
                    return Err("`--home-dir` requires a path argument".into());
                };

                if value.is_empty() {
                    return Err("`--home-dir` requires a non-empty path".into());
                }

                home_dir = Some(PathBuf::from(value));
                continue;
            }

            return Err(format!("unknown argument: {}", PathBuf::from(arg).display()));
        }

        Ok(Self { home_dir })
    }

    pub fn home_dir(&self) -> Option<&Path> {
        self.home_dir.as_deref()
    }

    pub fn apply_to_process_environment(&self) -> Result<(), std::io::Error> {
        let Some(home_dir) = self.home_dir() else {
            return Ok(());
        };

        fs::create_dir_all(home_dir)?;

        // This runs before Tauri startup and before the process spawns worker threads.
        unsafe {
            std::env::set_var("HOME", home_dir);
            std::env::set_var("USERPROFILE", home_dir);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeOptions;
    use std::{ffi::OsString, path::Path};

    #[test]
    fn parses_home_dir_with_equals_syntax() {
        let options =
            RuntimeOptions::from_args([OsString::from("--home-dir=/tmp/liveletters-a")]).unwrap();

        assert_eq!(options.home_dir(), Some(Path::new("/tmp/liveletters-a")));
    }

    #[test]
    fn parses_home_dir_with_separate_value() {
        let options = RuntimeOptions::from_args([
            OsString::from("--home-dir"),
            OsString::from("/tmp/liveletters-b"),
        ])
        .unwrap();

        assert_eq!(options.home_dir(), Some(Path::new("/tmp/liveletters-b")));
    }

    #[test]
    fn rejects_missing_home_dir_value() {
        let error = RuntimeOptions::from_args([OsString::from("--home-dir")]).unwrap_err();

        assert!(error.contains("requires a path"));
    }

    #[test]
    fn rejects_unknown_argument() {
        let error = RuntimeOptions::from_args([OsString::from("--verbose")]).unwrap_err();

        assert!(error.contains("unknown argument"));
    }
}
