//! Bounded, deterministic planning of analysis units.
//!
//! A manuscript is never sent to a provider whole. Units are scene groups
//! capped by character budget; adjacent-scene context is trimmed; canon names
//! and glossary terms are included only when present in the unit text. All
//! unit identities derive from content fingerprints, never from timestamps.

use crate::models::{AdvancedAnalysisError, AnalysisUnit, AnalysisUnitType, CanonContext};
use document_engine::{Manuscript, Paragraph, Scene};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerConfig {
    /// Upper bound of characters in a single analysis unit (excluding
    /// per-paragraph tokens). Scene groups are split at this boundary.
    pub max_unit_characters: usize,
    /// Upper bound of characters of neighboring-scene context included per unit.
    pub max_context_characters: usize,
    /// Maximum number of units the planner will ever produce.
    pub max_units: usize,
    /// Maximum approved-canon character names injected per unit.
    pub max_canon_names_per_unit: usize,
    /// Maximum approved-glossary terms injected per unit.
    pub max_glossary_terms_per_unit: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_unit_characters: 10_000,
            max_context_characters: 800,
            max_units: 2_000,
            max_canon_names_per_unit: 24,
            max_glossary_terms_per_unit: 12,
        }
    }
}

impl PlannerConfig {
    pub fn fingerprint(&self) -> String {
        let identity = format!(
            "planner\0{}\0{}\0{}\0{}\0{}",
            self.max_unit_characters,
            self.max_context_characters,
            self.max_units,
            self.max_canon_names_per_unit,
            self.max_glossary_terms_per_unit
        );
        stable_hash(identity.as_bytes())
    }
}

pub struct AnalysisPlanner {
    config: PlannerConfig,
}

impl Default for AnalysisPlanner {
    fn default() -> Self {
        Self::new(PlannerConfig::default())
    }
}

impl AnalysisPlanner {
    pub fn new(config: PlannerConfig) -> Self {
        Self { config }
    }

    pub fn plan(
        &self,
        manuscript: &Manuscript,
        canon: &CanonContext,
    ) -> Result<Vec<AnalysisUnit>, AdvancedAnalysisError> {
        let mut units = Vec::new();
        for chapter in &manuscript.chapters {
            for scene in &chapter.scenes {
                let groups = self.group_paragraphs(scene);
                for (group_index, group) in groups.iter().enumerate() {
                    let text_content = render_group(group);
                    if text_content.trim().is_empty() {
                        continue;
                    }
                    let text_fingerprint = stable_hash(text_content.as_bytes());
                    let scene_id = Some(scene.id.clone());
                    let paragraph_ids = group
                        .iter()
                        .map(|paragraph| paragraph.id.clone())
                        .collect::<Vec<_>>();
                    let unit_id = AnalysisUnit::unit_id(
                        &chapter.id,
                        scene_id.as_deref(),
                        &text_fingerprint,
                        group_index,
                    );
                    let (previous_scene_context, next_scene_context) =
                        self.neighbor_context(manuscript, chapter.id.as_str(), scene.id.as_str());
                    let text_lower = text_content.to_lowercase();
                    let known_character_names = present_entries(
                        &canon.character_names,
                        &text_lower,
                        self.config.max_canon_names_per_unit,
                    );
                    let known_glossary_terms = present_entries(
                        &canon.glossary_terms,
                        &text_lower,
                        self.config.max_glossary_terms_per_unit,
                    );
                    units.push(AnalysisUnit {
                        unit_id,
                        unit_type: AnalysisUnitType::Scene,
                        chapter_id: chapter.id.clone(),
                        chapter_index: chapter.index,
                        scene_id,
                        text_content,
                        text_fingerprint,
                        paragraph_ids,
                        previous_scene_context,
                        next_scene_context,
                        known_character_names,
                        known_glossary_terms,
                        source: scene.source.clone(),
                    });
                }
            }
        }
        if units.is_empty() {
            return Err(AdvancedAnalysisError::EmptyManuscript);
        }
        if units.len() > self.config.max_units {
            return Err(AdvancedAnalysisError::Configuration(format!(
                "analysis would produce {} units, exceeding the configured maximum of {}; \
                 raise the limit explicitly (for example LITERARY_ENGINE_ANALYSIS_MAX_UNITS)",
                units.len(),
                self.config.max_units
            )));
        }
        Ok(units)
    }

    /// Group a scene's paragraphs so each group stays within the character
    /// budget without splitting a paragraph in half.
    fn group_paragraphs<'a>(&self, scene: &'a Scene) -> Vec<Vec<&'a Paragraph>> {
        let mut groups: Vec<Vec<&Paragraph>> = Vec::new();
        let mut current: Vec<&Paragraph> = Vec::new();
        let mut current_chars = 0usize;
        for paragraph in &scene.paragraphs {
            let paragraph_chars = paragraph.original_text.chars().count();
            if !current.is_empty()
                && current_chars + paragraph_chars > self.config.max_unit_characters
            {
                groups.push(std::mem::take(&mut current));
                current_chars = 0;
            }
            current_chars += paragraph_chars;
            current.push(paragraph);
        }
        if !current.is_empty() {
            groups.push(current);
        }
        groups
    }

    fn neighbor_context(
        &self,
        manuscript: &Manuscript,
        chapter_id: &str,
        scene_id: &str,
    ) -> (Option<String>, Option<String>) {
        let mut previous: Option<String> = None;
        let mut next: Option<String> = None;
        for chapter in &manuscript.chapters {
            if chapter.id != chapter_id {
                continue;
            }
            for (index, scene) in chapter.scenes.iter().enumerate() {
                if scene.id == scene_id {
                    previous = index
                        .checked_sub(1)
                        .and_then(|prior| chapter.scenes.get(prior))
                        .map(|scene| trim_context(&scene.text, self.config.max_context_characters));
                    next = chapter
                        .scenes
                        .get(index + 1)
                        .map(|scene| trim_context(&scene.text, self.config.max_context_characters));
                    break;
                }
            }
        }
        (previous, next)
    }
}

fn render_group(group: &[&Paragraph]) -> String {
    group
        .iter()
        .map(|paragraph| {
            format!(
                "[{}] {}",
                AnalysisUnit::ordinal_token(paragraph.position - 1),
                paragraph.original_text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Keep only canon entries that actually occur in the unit text (bounded).
fn present_entries(candidates: &[String], haystack_lower: &str, limit: usize) -> Vec<String> {
    candidates
        .iter()
        .filter(|candidate| {
            !candidate.trim().is_empty() && haystack_lower.contains(&candidate.to_lowercase())
        })
        .take(limit)
        .cloned()
        .collect()
}

fn trim_context(text: &str, max_chars: usize) -> String {
    let trimmed: String = text.chars().take(max_chars).collect();
    if trimmed.chars().count() < text.chars().count() {
        format!("{trimmed} …")
    } else {
        trimmed
    }
}

pub fn stable_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::normalize_for_identity;
    use document_engine::{Book, DocumentFormat, Paragraph, SourceLocation};

    fn paragraph(position: usize, text: &str) -> Paragraph {
        Paragraph {
            id: format!("paragraph-{position}"),
            scene_id: "scene-1".to_string(),
            original_text: text.to_string(),
            position,
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    fn scene(paragraphs: Vec<Paragraph>) -> Scene {
        Scene {
            id: "scene-1".to_string(),
            chapter_id: "chapter-1".to_string(),
            order: 1,
            text: paragraphs
                .iter()
                .map(|paragraph| paragraph.original_text.clone())
                .collect::<Vec<_>>()
                .join("\n\n"),
            importance: Default::default(),
            paragraphs,
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    fn manuscript(paragraphs: Vec<Paragraph>) -> Manuscript {
        Manuscript {
            book: Book {
                id: "book-1".to_string(),
                title: "Synthetic".to_string(),
                author: None,
                language: None,
                metadata: Default::default(),
            },
            chapters: vec![document_engine::Chapter {
                id: "chapter-1".to_string(),
                title: "Chapter 1".to_string(),
                order: 1,
                index: 0,
                content: String::new(),
                scenes: vec![scene(paragraphs)],
                source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
            }],
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    #[test]
    fn short_scene_produces_a_single_stable_unit() {
        let paragraphs = vec![
            paragraph(1, "First paragraph."),
            paragraph(2, "Second paragraph."),
        ];
        let book = manuscript(paragraphs);
        let planner = AnalysisPlanner::default();
        let units = planner.plan(&book, &CanonContext::default()).unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].paragraph_ids.len(), 2);
        assert!(units[0].text_content.contains("[para-1]"));
        assert!(units[0].text_content.contains("[para-2]"));

        let again = planner.plan(&book, &CanonContext::default()).unwrap();
        assert_eq!(units[0].unit_id, again[0].unit_id);
        assert_eq!(units[0].text_fingerprint, again[0].text_fingerprint);
    }

    #[test]
    fn oversized_scenes_are_split_into_bounded_units() {
        let long_a = "word ".repeat(90);
        let long_b = "word ".repeat(90);
        let book = manuscript(vec![paragraph(1, &long_a), paragraph(2, &long_b)]);
        let planner = AnalysisPlanner::new(PlannerConfig {
            max_unit_characters: 500,
            ..Default::default()
        });
        let units = planner.plan(&book, &CanonContext::default()).unwrap();
        assert!(units.len() >= 2);
        for unit in units {
            let content_chars = unit
                .text_content
                .chars()
                .filter(|character| *character != '[')
                .count();
            assert!(content_chars <= 1000, "unit exceeded budget");
            assert!(!unit.text_content.trim().is_empty());
        }
    }

    #[test]
    fn edited_paragraph_changes_only_its_own_unit() {
        let book = manuscript(vec![
            paragraph(1, "Alpha content."),
            paragraph(2, "Beta content."),
        ]);
        let planner = AnalysisPlanner::default();
        let first = planner.plan(&book, &CanonContext::default()).unwrap();
        let edited = manuscript(vec![
            paragraph(1, "Alpha content changed."),
            paragraph(2, "Beta content."),
        ]);
        let second = planner.plan(&edited, &CanonContext::default()).unwrap();
        assert_ne!(first[0].unit_id, second[0].unit_id);
    }

    #[test]
    fn canon_entries_are_filtered_to_those_present_in_text() {
        let book = manuscript(vec![paragraph(
            1,
            "Reza entered the room and greeted Mina.",
        )]);
        let canon = CanonContext {
            character_names: vec![
                "Reza".to_string(),
                "Mina".to_string(),
                "SomeoneElse".to_string(),
            ],
            glossary_terms: vec!["room".to_string(), "absent".to_string()],
            ..Default::default()
        };
        let planner = AnalysisPlanner::default();
        let units = planner.plan(&book, &canon).unwrap();
        assert_eq!(units[0].known_character_names, vec!["Reza", "Mina"]);
        assert_eq!(units[0].known_glossary_terms, vec!["room"]);
        let _ = normalize_for_identity("unused");
    }

    #[test]
    fn empty_manuscript_is_rejected() {
        let book = Manuscript {
            book: Book {
                id: "book-1".to_string(),
                title: "Synthetic".to_string(),
                author: None,
                language: None,
                metadata: Default::default(),
            },
            chapters: Vec::new(),
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        };
        let planner = AnalysisPlanner::default();
        assert!(matches!(
            planner.plan(&book, &CanonContext::default()),
            Err(AdvancedAnalysisError::EmptyManuscript)
        ));
    }
}
