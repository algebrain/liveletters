use serde::{Deserialize, Serialize};

/// Защитные лимиты MIME-разбора. Каждое поле несёт собственный serde-default:
/// частичный файл `users/<name>/config.toml` дополняется кодовыми значениями
/// для отсутствующих ключей.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MimeLimits {
    #[serde(default = "default_max_raw_email_bytes")]
    pub max_raw_email_bytes: usize,
    #[serde(default = "default_max_human_bytes")]
    pub max_human_bytes: usize,
    #[serde(default = "default_max_json_bytes")]
    pub max_json_bytes: usize,
    #[serde(default = "default_max_parts")]
    pub max_parts: usize,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

impl Default for MimeLimits {
    fn default() -> Self {
        Self {
            // These v1 defaults assume no post/comment attachments yet.
            // When attachments get a liveletters.json manifest, split raw
            // email and attachment limits instead of only raising this value.
            max_raw_email_bytes: default_max_raw_email_bytes(),
            max_human_bytes: default_max_human_bytes(),
            max_json_bytes: default_max_json_bytes(),
            max_parts: default_max_parts(),
            max_depth: default_max_depth(),
        }
    }
}

fn default_max_raw_email_bytes() -> usize {
    10 * 1024 * 1024
}
fn default_max_human_bytes() -> usize {
    1024 * 1024
}
fn default_max_json_bytes() -> usize {
    1024 * 1024
}
fn default_max_parts() -> usize {
    8
}
fn default_max_depth() -> usize {
    2
}
