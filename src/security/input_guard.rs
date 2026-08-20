use std::path::Path;

pub struct InputGuard;

impl InputGuard {
    pub fn is_supported_file(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("pdf") | Some("epub") | Some("docx") | Some("txt")
        )
    }

    pub fn is_safe_size(size_bytes: u64) -> bool {
        // Large novels, collections, and archival documents may require more space.
        // Keep a bounded limit while supporting real-world long-form translation.
        const MAX_BYTES: u64 = 3 * 1024 * 1024 * 1024;
        size_bytes <= MAX_BYTES
    }
}
