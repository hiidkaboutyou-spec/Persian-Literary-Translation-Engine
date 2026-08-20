//! Chapter and scene chunking foundation.
//! This prepares documents for context-aware translation.

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub index: usize,
    pub content: String,
}

pub fn split_into_chunks(text: &str, max_chars: usize) -> Vec<TextChunk> {
    let size = max_chars.max(1);
    text.as_bytes()
        .chunks(size)
        .enumerate()
        .map(|(index, chunk)| TextChunk {
            index,
            content: String::from_utf8_lossy(chunk).to_string(),
        })
        .collect()
}
