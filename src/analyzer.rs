#[derive(Debug, Default)]
pub struct SceneAnalysis {
    pub summary: String,
    pub characters: Vec<String>,
    pub emotional_notes: Vec<String>,
}

pub fn analyze_scene(text: &str) -> SceneAnalysis {
    SceneAnalysis {
        summary: text.lines().next().unwrap_or_default().to_string(),
        characters: Vec::new(),
        emotional_notes: Vec::new(),
    }
}
