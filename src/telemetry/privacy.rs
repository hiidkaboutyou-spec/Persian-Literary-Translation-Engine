pub struct TelemetryPrivacy;

impl TelemetryPrivacy {
    pub fn sanitize_label(value: &str) -> String {
        value
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .take(64)
            .collect()
    }

    pub fn is_sensitive_field(field: &str) -> bool {
        let field = field.to_lowercase();
        [
            "text",
            "content",
            "manuscript",
            "translation",
            "api_key",
            "token",
            "secret",
        ]
        .iter()
        .any(|item| field.contains(item))
    }
}
