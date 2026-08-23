//! Chapter extraction for prose documents.

use crate::models::Chapter;

fn is_heading(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() > 120 {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("chapter ")
        || lower.starts_with("part ")
        || trimmed.starts_with("فصل ")
        || trimmed.starts_with("بخش ")
}

pub fn split_into_chapters(text: &str) -> Vec<Chapter> {
    let mut chapters = Vec::new();
    let mut current_title = String::new();
    let mut current_lines: Vec<&str> = Vec::new();

    for line in text.lines() {
        if is_heading(line) {
            if !current_lines.iter().all(|line| line.trim().is_empty()) {
                let index = chapters.len();
                chapters.push(Chapter::translated(
                    index,
                    if current_title.is_empty() {
                        format!("Chapter {}", index + 1)
                    } else {
                        current_title.clone()
                    },
                    current_lines.join("\n").trim().to_string(),
                ));
            }
            current_title = line.trim().to_string();
            current_lines.clear();
        } else {
            current_lines.push(line);
        }
    }

    if !current_lines.iter().all(|line| line.trim().is_empty()) || !current_title.is_empty() {
        let index = chapters.len();
        chapters.push(Chapter::translated(
            index,
            if current_title.is_empty() {
                format!("Chapter {}", index + 1)
            } else {
                current_title
            },
            current_lines.join("\n").trim().to_string(),
        ));
    }

    if chapters.is_empty() && !text.trim().is_empty() {
        chapters.push(Chapter::translated(0, "Chapter 1", text.trim()));
    }

    chapters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_prose_stays_in_one_chapter() {
        let chapters = split_into_chapters("First paragraph.\n\nSecond paragraph.");
        assert_eq!(chapters.len(), 1);
        assert!(chapters[0].content.contains("Second paragraph"));
    }

    #[test]
    fn english_headings_create_chapters() {
        let chapters = split_into_chapters("Chapter 1\nHello\nChapter 2\nWorld");
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter 1");
        assert_eq!(chapters[1].content, "World");
    }

    #[test]
    fn persian_headings_create_chapters() {
        let chapters = split_into_chapters("فصل ۱\nسلام\nفصل ۲\nخداحافظ");
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[1].title, "فصل ۲");
    }
}
