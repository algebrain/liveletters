#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MimeLimits {
    pub max_raw_email_bytes: usize,
    pub max_human_bytes: usize,
    pub max_json_bytes: usize,
    pub max_parts: usize,
    pub max_depth: usize,
}

impl Default for MimeLimits {
    fn default() -> Self {
        Self {
            max_raw_email_bytes: 10 * 1024 * 1024,
            max_human_bytes: 1024 * 1024,
            max_json_bytes: 1024 * 1024,
            max_parts: 8,
            max_depth: 2,
        }
    }
}
