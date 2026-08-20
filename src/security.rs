use std::path::Path;

pub struct SecurityValidator;

impl SecurityValidator {
    pub fn validate_input_path(path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    pub fn contains_secret_like_data(value: &str) -> bool {
        let patterns = ["sk-", "api_key", "secret", "token"];
        patterns.iter().any(|p| value.to_lowercase().contains(p))
    }
}
