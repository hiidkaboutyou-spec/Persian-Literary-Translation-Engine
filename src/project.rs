use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationProject {
    pub name: String,
    pub source_language: String,
    pub target_language: String,
    pub chapters: Vec<String>,
}

impl TranslationProject {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            source_language: "English".to_string(),
            target_language: "Persian".to_string(),
            chapters: Vec::new(),
        }
    }

    pub fn add_chapter(&mut self, chapter: &str) {
        self.chapters.push(chapter.to_string());
    }
}
