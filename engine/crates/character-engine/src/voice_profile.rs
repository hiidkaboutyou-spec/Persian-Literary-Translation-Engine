#[derive(Debug, Clone)]
pub struct VoiceProfile {
    pub character_name: String,
    pub speech_style: String,
    pub vocabulary_notes: Vec<String>,
    pub emotional_patterns: Vec<String>,
}

impl VoiceProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            character_name: name.into(),
            speech_style: String::new(),
            vocabulary_notes: Vec::new(),
            emotional_patterns: Vec::new(),
        }
    }
}
