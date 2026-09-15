use std::cmp::Ordering;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::context::MemoryContextConfig;
use crate::glossary::Glossary;
use crate::{RetrievalConfig, TranslationMemory};

pub const CONTEXT_PACKET_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextAuthority {
    Advisory,
    Inferred,
    Deterministic,
    Canonical,
    HumanApproved,
}

impl ContextAuthority {
    fn priority(self) -> u8 {
        match self {
            Self::Advisory => 1,
            Self::Inferred => 2,
            Self::Deterministic => 3,
            Self::Canonical => 4,
            Self::HumanApproved => 5,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Advisory => "advisory",
            Self::Inferred => "inferred",
            Self::Deterministic => "deterministic",
            Self::Canonical => "canonical",
            Self::HumanApproved => "human-approved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextKind {
    Glossary,
    TranslationDecision,
    Character,
    Relationship,
    LocalContinuity,
    SceneSummary,
    ChapterSummary,
    ArcSummary,
    BookSummary,
    TranslationMemory,
    Reference,
    Advisory,
}

impl ContextKind {
    fn priority(self) -> u8 {
        match self {
            Self::Glossary => 100,
            Self::TranslationDecision => 95,
            Self::Character => 90,
            Self::Relationship => 85,
            Self::LocalContinuity => 80,
            Self::SceneSummary => 75,
            Self::ChapterSummary => 70,
            Self::ArcSummary => 60,
            Self::BookSummary => 50,
            Self::TranslationMemory => 45,
            Self::Reference => 35,
            Self::Advisory => 20,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Glossary => "glossary",
            Self::TranslationDecision => "translation-decision",
            Self::Character => "character",
            Self::Relationship => "relationship",
            Self::LocalContinuity => "local-continuity",
            Self::SceneSummary => "scene-summary",
            Self::ChapterSummary => "chapter-summary",
            Self::ArcSummary => "arc-summary",
            Self::BookSummary => "book-summary",
            Self::TranslationMemory => "translation-memory",
            Self::Reference => "reference",
            Self::Advisory => "advisory",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextCandidate {
    pub id: String,
    pub kind: ContextKind,
    pub authority: ContextAuthority,
    pub text: String,
    pub reason: String,
    pub relevance: f32,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

impl ContextCandidate {
    pub fn new(
        id: impl Into<String>,
        kind: ContextKind,
        authority: ContextAuthority,
        text: impl Into<String>,
        reason: impl Into<String>,
        relevance: f32,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            authority,
            text: text.into(),
            reason: reason.into(),
            relevance: sanitize_score(relevance),
            evidence_ids: Vec::new(),
        }
    }

    pub fn with_evidence_ids(mut self, evidence_ids: impl IntoIterator<Item = String>) -> Self {
        self.evidence_ids = evidence_ids.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextItem {
    pub id: String,
    pub kind: ContextKind,
    pub authority: ContextAuthority,
    pub text: String,
    pub reason: String,
    pub relevance: f32,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPacketBudget {
    pub max_chars: usize,
    pub max_items: usize,
    pub max_item_chars: usize,
}

impl Default for ContextPacketBudget {
    fn default() -> Self {
        Self {
            max_chars: 8_000,
            max_items: 28,
            max_item_chars: 1_800,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPacketConfig {
    pub budget: ContextPacketBudget,
    pub min_relevance: f32,
}

impl Default for ContextPacketConfig {
    fn default() -> Self {
        Self {
            budget: ContextPacketBudget::default(),
            min_relevance: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPacketV2 {
    pub schema_version: u32,
    pub unit_id: String,
    pub source_fingerprint: String,
    pub canon_fingerprint: String,
    pub packet_fingerprint: String,
    pub rendered_context: String,
    pub used_chars: usize,
    pub selected_count: usize,
    pub excluded_count: usize,
    pub truncated: bool,
    pub items: Vec<ContextItem>,
}

/// Build a deterministic, provenance-carrying context packet from candidates owned
/// by the surrounding domain engines. This module does not infer character canon,
/// relationships, summaries, or translation decisions; it only prioritizes and
/// budgets evidence supplied to it.
pub fn build_context_packet_v2(
    unit_id: &str,
    source_text: &str,
    candidates: &[ContextCandidate],
    config: &ContextPacketConfig,
) -> ContextPacketV2 {
    let source_fingerprint = sha256_hex(source_text.as_bytes());
    let canon_fingerprint = fingerprint_candidates(candidates);

    if source_text.trim().is_empty() || config.budget.max_chars == 0 || config.budget.max_items == 0
    {
        return finalize_packet(
            unit_id,
            source_fingerprint,
            canon_fingerprint,
            config,
            Vec::new(),
            String::new(),
            0,
            candidates.len(),
            false,
        );
    }

    let mut eligible = candidates
        .iter()
        .filter(|candidate| {
            !candidate.id.trim().is_empty()
                && !candidate.text.trim().is_empty()
                && sanitize_score(candidate.relevance) >= config.min_relevance.clamp(0.0, 1.0)
        })
        .cloned()
        .collect::<Vec<_>>();

    eligible.sort_by(compare_candidates);

    let mut seen_ids = HashSet::new();
    let mut selected = Vec::new();
    let mut rendered = String::new();
    let mut excluded_count = candidates.len().saturating_sub(eligible.len());
    let mut truncated = false;

    for candidate in eligible {
        if selected.len() >= config.budget.max_items {
            excluded_count += 1;
            truncated = true;
            continue;
        }
        if !seen_ids.insert(candidate.id.clone()) {
            excluded_count += 1;
            continue;
        }

        let mut item_text = truncate_chars(candidate.text.trim(), config.budget.max_item_chars);
        let item_truncated = item_text.chars().count() < candidate.text.trim().chars().count();
        if item_truncated {
            item_text.push_str(" …");
        }

        let header = format!(
            "[{} | {} | id={} | relevance={:.3}]\nReason: {}\n",
            candidate.kind.label(),
            candidate.authority.label(),
            candidate.id,
            sanitize_score(candidate.relevance),
            candidate.reason.trim()
        );
        let separator = if rendered.is_empty() { "" } else { "\n\n" };
        let fixed_chars = separator.chars().count() + header.chars().count();
        let used_chars = rendered.chars().count();
        let remaining = config.budget.max_chars.saturating_sub(used_chars);

        if remaining <= fixed_chars {
            excluded_count += 1;
            truncated = true;
            continue;
        }

        let available_text_chars = remaining - fixed_chars;
        let fitted_text = truncate_chars(&item_text, available_text_chars);
        if fitted_text.trim().is_empty() {
            excluded_count += 1;
            truncated = true;
            continue;
        }
        let budget_truncated = fitted_text.chars().count() < item_text.chars().count();
        if budget_truncated {
            truncated = true;
        }

        rendered.push_str(separator);
        rendered.push_str(&header);
        rendered.push_str(&fitted_text);

        selected.push(ContextItem {
            id: candidate.id,
            kind: candidate.kind,
            authority: candidate.authority,
            text: fitted_text,
            reason: candidate.reason.trim().to_string(),
            relevance: sanitize_score(candidate.relevance),
            evidence_ids: candidate.evidence_ids,
            truncated: item_truncated || budget_truncated,
        });
    }

    finalize_packet(
        unit_id,
        source_fingerprint,
        canon_fingerprint,
        config,
        selected,
        rendered,
        candidates.len(),
        excluded_count,
        truncated,
    )
}

/// Convert the existing glossary and translation-memory retrieval into V2
/// candidates without changing their current storage or ranking contracts.
pub fn native_memory_candidates(
    source_text: &str,
    memory: &TranslationMemory,
    glossary: &Glossary,
    config: &MemoryContextConfig,
) -> Vec<ContextCandidate> {
    if source_text.trim().is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for entry in glossary
        .relevant_to_text(source_text)
        .into_iter()
        .take(config.max_glossary_entries)
    {
        let text = if entry.context.trim().is_empty() {
            format!("{} => {}", entry.source_term, entry.preferred_translation)
        } else {
            format!(
                "{} => {} [{}]",
                entry.source_term,
                entry.preferred_translation,
                entry.context.trim()
            )
        };
        let id = stable_evidence_id(
            "glossary",
            &[
                &entry.source_term,
                &entry.preferred_translation,
                &entry.context,
            ],
        );
        candidates.push(ContextCandidate::new(
            id,
            ContextKind::Glossary,
            ContextAuthority::Canonical,
            text,
            "source term is present in the current translation unit",
            1.0,
        ));
    }

    let retrieval_config = RetrievalConfig {
        max_results: config.max_memory_hits,
        min_score: config.min_memory_score,
        ..RetrievalConfig::default()
    };
    for hit in memory.search_with_config(source_text, &retrieval_config) {
        let entry = hit.entry;
        let text = if entry.context.trim().is_empty() {
            format!("{:?} => {:?}", entry.source, entry.translation)
        } else {
            format!(
                "{:?} => {:?} | context: {}",
                entry.source,
                entry.translation,
                entry.context.trim()
            )
        };
        let id = stable_evidence_id(
            "translation-memory",
            &[&entry.source, &entry.translation, &entry.context],
        );
        candidates.push(ContextCandidate::new(
            id,
            ContextKind::TranslationMemory,
            ContextAuthority::Deterministic,
            text,
            "deterministic translation-memory retrieval matched the current unit",
            hit.score,
        ));
    }

    candidates
}

pub fn stable_evidence_id(namespace: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    for part in parts {
        hasher.update([0x1f]);
        hasher.update(part.as_bytes());
    }
    format!("{}-{}", namespace, hex_digest(hasher.finalize()))
}

fn compare_candidates(a: &ContextCandidate, b: &ContextCandidate) -> Ordering {
    b.authority
        .priority()
        .cmp(&a.authority.priority())
        .then_with(|| b.kind.priority().cmp(&a.kind.priority()))
        .then_with(|| sanitize_score(b.relevance).total_cmp(&sanitize_score(a.relevance)))
        .then_with(|| a.id.cmp(&b.id))
}

fn fingerprint_candidates(candidates: &[ContextCandidate]) -> String {
    let mut canonical = candidates.to_vec();
    canonical.iter_mut().for_each(|candidate| {
        candidate.relevance = sanitize_score(candidate.relevance);
        candidate.reason = candidate.reason.trim().to_string();
        candidate.text = candidate.text.trim().to_string();
        candidate.evidence_ids.sort();
    });
    canonical.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.kind.cmp(&b.kind)));
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    sha256_hex(&bytes)
}

#[allow(clippy::too_many_arguments)]
fn finalize_packet(
    unit_id: &str,
    source_fingerprint: String,
    canon_fingerprint: String,
    config: &ContextPacketConfig,
    items: Vec<ContextItem>,
    rendered_context: String,
    candidate_count: usize,
    excluded_count: usize,
    truncated: bool,
) -> ContextPacketV2 {
    #[derive(Serialize)]
    struct FingerprintPayload<'a> {
        schema_version: u32,
        unit_id: &'a str,
        source_fingerprint: &'a str,
        canon_fingerprint: &'a str,
        config: &'a ContextPacketConfig,
        items: &'a [ContextItem],
        rendered_context: &'a str,
    }

    let payload = FingerprintPayload {
        schema_version: CONTEXT_PACKET_SCHEMA_VERSION,
        unit_id,
        source_fingerprint: &source_fingerprint,
        canon_fingerprint: &canon_fingerprint,
        config,
        items: &items,
        rendered_context: &rendered_context,
    };
    let packet_fingerprint = sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default());
    let used_chars = rendered_context.chars().count();

    ContextPacketV2 {
        schema_version: CONTEXT_PACKET_SCHEMA_VERSION,
        unit_id: unit_id.to_string(),
        source_fingerprint,
        canon_fingerprint,
        packet_fingerprint,
        rendered_context,
        used_chars,
        selected_count: items.len(),
        excluded_count: excluded_count.max(candidate_count.saturating_sub(items.len())),
        truncated,
        items,
    }
}

fn sanitize_score(score: f32) -> f32 {
    if score.is_finite() {
        score.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_digest(hasher.finalize())
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::GlossaryEntry;
    use crate::MemoryEntry;
    use std::collections::BTreeMap;

    fn candidate(
        id: &str,
        kind: ContextKind,
        authority: ContextAuthority,
        text: &str,
        relevance: f32,
    ) -> ContextCandidate {
        ContextCandidate::new(id, kind, authority, text, "test evidence", relevance)
    }

    #[test]
    fn packet_is_deterministic_independent_of_input_order() {
        let a = candidate(
            "character-a",
            ContextKind::Character,
            ContextAuthority::Canonical,
            "Character A speaks tersely.",
            0.8,
        );
        let b = candidate(
            "tm-b",
            ContextKind::TranslationMemory,
            ContextAuthority::Deterministic,
            "I missed you => دلم برات تنگ شده بود",
            0.95,
        );
        let config = ContextPacketConfig::default();
        let first = build_context_packet_v2("u1", "I missed you", &[a.clone(), b.clone()], &config);
        let second = build_context_packet_v2("u1", "I missed you", &[b, a], &config);

        assert_eq!(first.packet_fingerprint, second.packet_fingerprint);
        assert_eq!(first.rendered_context, second.rendered_context);
    }

    #[test]
    fn human_and_canonical_evidence_outrank_soft_memory_under_budget() {
        let candidates = vec![
            candidate(
                "tm",
                ContextKind::TranslationMemory,
                ContextAuthority::Deterministic,
                "soft memory evidence that is intentionally long",
                1.0,
            ),
            candidate(
                "term",
                ContextKind::Glossary,
                ContextAuthority::Canonical,
                "High Warlock => جادوگر اعظم",
                1.0,
            ),
            candidate(
                "decision",
                ContextKind::TranslationDecision,
                ContextAuthority::HumanApproved,
                "Use محاوره‌ای register between these speakers.",
                0.2,
            ),
        ];
        let config = ContextPacketConfig {
            budget: ContextPacketBudget {
                max_chars: 260,
                max_items: 2,
                max_item_chars: 120,
            },
            min_relevance: 0.0,
        };
        let packet = build_context_packet_v2("u1", "source", &candidates, &config);

        assert_eq!(packet.selected_count, 2);
        assert_eq!(packet.items[0].id, "decision");
        assert_eq!(packet.items[1].id, "term");
        assert!(!packet.items.iter().any(|item| item.id == "tm"));
    }

    #[test]
    fn fingerprint_changes_when_source_or_context_changes() {
        let base = candidate(
            "term",
            ContextKind::Glossary,
            ContextAuthority::Canonical,
            "Portal => پرتال",
            1.0,
        );
        let config = ContextPacketConfig::default();
        let first = build_context_packet_v2(
            "u1",
            "Open the Portal",
            std::slice::from_ref(&base),
            &config,
        );
        let second = build_context_packet_v2(
            "u1",
            "Close the Portal",
            std::slice::from_ref(&base),
            &config,
        );
        let changed = candidate(
            "term",
            ContextKind::Glossary,
            ContextAuthority::Canonical,
            "Portal => دروازه",
            1.0,
        );
        let third = build_context_packet_v2("u1", "Open the Portal", &[changed], &config);

        assert_ne!(first.packet_fingerprint, second.packet_fingerprint);
        assert_ne!(first.packet_fingerprint, third.packet_fingerprint);
        assert_ne!(first.canon_fingerprint, third.canon_fingerprint);
    }

    #[test]
    fn oversized_unicode_evidence_is_truncated_safely() {
        let long = "سلام دنیا ".repeat(200);
        let packet = build_context_packet_v2(
            "u1",
            "source",
            &[candidate(
                "summary",
                ContextKind::ChapterSummary,
                ContextAuthority::Canonical,
                &long,
                0.8,
            )],
            &ContextPacketConfig {
                budget: ContextPacketBudget {
                    max_chars: 220,
                    max_items: 4,
                    max_item_chars: 140,
                },
                min_relevance: 0.0,
            },
        );

        assert!(packet.truncated);
        assert!(packet.items[0].truncated);
        assert!(packet.used_chars <= 220);
        assert!(packet
            .rendered_context
            .is_char_boundary(packet.rendered_context.len()));
    }

    #[test]
    fn native_candidates_bridge_existing_glossary_and_memory() {
        let mut glossary = Glossary::default();
        glossary.add(GlossaryEntry {
            source_term: "Portal".into(),
            preferred_translation: "پرتال".into(),
            context: "magic doorway".into(),
        });
        let mut memory = TranslationMemory::new();
        memory.add(MemoryEntry::new(
            "He opened the Portal".into(),
            "پرتال را باز کرد".into(),
            "narration".into(),
        ));

        let candidates = native_memory_candidates(
            "He opened the Portal",
            &memory,
            &glossary,
            &MemoryContextConfig::default(),
        );

        assert!(candidates
            .iter()
            .any(|item| item.kind == ContextKind::Glossary));
        assert!(candidates
            .iter()
            .any(|item| item.kind == ContextKind::TranslationMemory));
    }

    #[test]
    fn duplicate_ids_are_included_once() {
        let one = candidate(
            "same",
            ContextKind::Reference,
            ContextAuthority::Advisory,
            "first",
            0.8,
        );
        let two = candidate(
            "same",
            ContextKind::Reference,
            ContextAuthority::Advisory,
            "second",
            0.7,
        );
        let packet =
            build_context_packet_v2("u1", "source", &[two, one], &ContextPacketConfig::default());
        assert_eq!(packet.items.len(), 1);
        assert_eq!(packet.items[0].text, "first");
    }

    #[test]
    fn candidate_fingerprint_is_stable_for_evidence_order() {
        let a = candidate(
            "x",
            ContextKind::Reference,
            ContextAuthority::Advisory,
            "text",
            0.5,
        )
        .with_evidence_ids(["b".to_string(), "a".to_string()]);
        let b = candidate(
            "x",
            ContextKind::Reference,
            ContextAuthority::Advisory,
            "text",
            0.5,
        )
        .with_evidence_ids(["a".to_string(), "b".to_string()]);
        assert_eq!(fingerprint_candidates(&[a]), fingerprint_candidates(&[b]));
    }

    #[test]
    fn packet_kind_counts_can_be_derived_without_prompt_parsing() {
        let packet = build_context_packet_v2(
            "u1",
            "source",
            &[
                candidate(
                    "a",
                    ContextKind::Character,
                    ContextAuthority::Canonical,
                    "A",
                    1.0,
                ),
                candidate(
                    "b",
                    ContextKind::Relationship,
                    ContextAuthority::Canonical,
                    "B",
                    1.0,
                ),
            ],
            &ContextPacketConfig::default(),
        );
        let counts = packet.items.iter().fold(BTreeMap::new(), |mut map, item| {
            *map.entry(item.kind).or_insert(0usize) += 1;
            map
        });
        assert_eq!(counts[&ContextKind::Character], 1);
        assert_eq!(counts[&ContextKind::Relationship], 1);
    }
}
