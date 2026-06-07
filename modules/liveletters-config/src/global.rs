use serde::{Deserialize, Serialize};

use liveletters_log::LogConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub log: LogConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            log: LogConfig::default(),
        }
    }
}

fn default_schema_version() -> u32 {
    1
}
