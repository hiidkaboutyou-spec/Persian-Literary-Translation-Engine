//! Chapter and scene chunking foundation.
//! This prepares documents for context-aware translation.
//!
//! Splitting uses character boundaries (not bytes) so multi-byte UTF-8 sequences
//! such as Persian and Arabic text are never broken mid-character. Whitespace
//! boundaries are preferred when available.

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub index: usize,
    pub content: String,
}

pub fn split_into_chunks(text: &str, max_chars: usize) -> Vec<TextChunk> {
    let size = max_chars.max(1);
    let mut chunks = Vec::new();
    let mut remaining = text;

    while remaining.chars().count() > size {
        let hard_end = remaining
            .char_indices()
            .nth(size)
            .map(|(index, _)| index)
            .unwrap_or(remaining.len());
        let candidate = &remaining[..hard_end];
        let boundary = candidate
            .char_indices()
            .rev()
            .find(|(_, character)| character.is_whitespace())
            .map(|(index, character)| index + character.len_utf8())
            .filter(|index| *index > 0)
            .unwrap_or(hard_end);

        chunks.push(TextChunk {
            index: chunks.len(),
            content: remaining[..boundary].to_owned(),
        });
        remaining = &remaining[boundary..];
    }

    if !remaining.is_empty() {
        chunks.push(TextChunk {
            index: chunks.len(),
            content: remaining.to_owned(),
        });
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_ascii_text_at_whitespace() {
        let chunks = split_into_chunks("alpha beta gamma delta", 10);
        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        assert_eq!(texts, vec!["alpha ", "beta ", "gamma delta"]);
    }

    #[test]
    fn splits_persian_text_without_breaking_characters() {
        let text = "سلام دنیا — یک متن آزمایشی برای آزمایش شکستن متن";
        let chunks = split_into_chunks(text, 8);
        for chunk in &chunks {
            assert!(
                chunk.content.chars().count() <= 8,
                "chunk too large: {:?} ({} chars)",
                chunk.content,
                chunk.content.chars().count()
            );
        }
        let reassembled: String = chunks.iter().map(|c| c.content.as_str()).collect();
        assert_eq!(reassembled, text);
    }

    #[test]
    fn preserves_all_content_losslessly() {
        let text = "Hello 你好 مرحبا world";
        let chunks = split_into_chunks(text, 6);
        let reassembled: String = chunks.iter().map(|c| c.content.as_str()).collect();
        assert_eq!(reassembled, text);
    }

    #[test]
    fn single_chunk_for_short_text() {
        let chunks = split_into_chunks("short", 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "short");
        assert_eq!(chunks[0].index, 0);
    }

    #[test]
    fn chunk_indices_are_sequential() {
        let chunks = split_into_chunks("a b c d e f g h i j", 4);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn empty_text_produces_no_chunks() {
        let chunks = split_into_chunks("", 10);
        assert!(chunks.is_empty());
    }
}
