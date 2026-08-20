//! Chapter extraction foundation.
//! This prepares documents for literary translation workflows.

#[derive(Debug, Clone)]
pub struct Chapter {
    pub index: usize,
    pub title: String,
    pub content: String,
}

pub fn split_into_chapters(text: &str) -> Vec<Chapter> {
    text.split("\n\n")
        .enumerate()
        .filter(|(_, part)| !part.trim().is_empty())
        .map(|(index, part)| Chapter {
            index,
            title: format!("Chapter {}", index + 1),
            content: part.to_string(),
        })
        .collect()
}
