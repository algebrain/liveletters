mod backend;
mod dto;
mod errors;

pub use backend::BackendApp;
pub use dto::{CreatePostRequest, IncomingFailureDto, SyncStatusDto};
pub use errors::BackendError;

pub fn app_name() -> &'static str {
    "liveletters-rust-backend-app"
}

#[cfg(test)]
mod tests {
    use super::app_name;

    #[test]
    fn exposes_app_name() {
        assert_eq!(app_name(), "liveletters-rust-backend-app");
    }
}
