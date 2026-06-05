use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
        }
    }
}

fn default_schema_version() -> u32 {
    1
}
