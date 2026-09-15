use std::collections::HashMap;

use crate::context::MemoryContextConfig;
use crate::context_v2::{
    native_memory_candidates, stable_evidence_id, ContextAuthority, ContextCandidate, ContextKind,
};
use crate::glossary::Glossary;
use crate::retrieval::RetrievalConfig;
use crate::semantic::{
    reciprocal_rank_fusion, ReciprocalRankFusionConfig, SemanticCandidate, SemanticRerankRequest,
    SemanticSidecar,
};
use crate::TranslationMemory;

const MAX_SEMANTIC_CANDIDATES: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticRetrievalStatus {
    pub attempted: bool,
    pub used: bool,
    pub model: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HybridMemoryCandidates {
    pub candidates: Vec<ContextCandidate>,
    pub semantic: SemanticRetrievalStatus,
}

/// Retrieve glossary/TM evidence with deterministic ranking as the safety floor,
/// then optionally rerank a bounded TM pool through the semantic sidecar.
///
/// The sidecar is advisory only. Any semantic error returns the exact deterministic
/// candidate set rather than failing translation or mutating canon.
pub fn hybrid_memory_candidates(
    source_text: &str,
    memory: &TranslationMemory,
    glossary: &Glossary,
    config: &MemoryContextConfig,
    semantic_sidecar: Option<&SemanticSidecar>,
) -> HybridMemoryCandidates {
    let deterministic = native_memory_candidates(source_text, memory, glossary, config);
    let Some(sidecar) = semantic_sidecar else {
        return HybridMemoryCandidates {
            candidates: deterministic,
            semantic: SemanticRetrievalStatus::default(),
        };
    };

    let mut status = SemanticRetrievalStatus {
        attempted: true,
        ..SemanticRetrievalStatus::default()
    };

    if source_text.trim().is_empty() || config.max_memory_hits == 0 || memory.entries().is_empty() {
        return HybridMemoryCandidates {
            candidates: deterministic,
            semantic: status,
        };
    }

    // When the whole TM fits in the hard protocol bound, semantic retrieval gets
    // true corpus-wide recall. For larger memories, deterministic ranking supplies
    // the bounded candidate pool until a persistent vector index is justified by
    // benchmark evidence.
    let pool_limit = memory.entries().len().min(MAX_SEMANTIC_CANDIDATES);
    let retrieval_config = RetrievalConfig {
        max_results: pool_limit,
        min_score: 0.0,
        diversity_threshold: 1.0,
        deduplicate_translations: false,
        ..RetrievalConfig::default()
    };
    let pool = memory.search_with_config(source_text, &retrieval_config);
    if pool.is_empty() {
        return HybridMemoryCandidates {
            candidates: deterministic,
            semantic: status,
        };
    }

    let mut by_id = HashMap::with_capacity(pool.len());
    let mut semantic_candidates = Vec::with_capacity(pool.len());
    let mut deterministic_ids = Vec::with_capacity(pool.len());

    for hit in &pool {
        let entry = hit.entry;
        let id = stable_evidence_id(
            "translation-memory",
            &[&entry.source, &entry.translation, &entry.context],
        );
        let semantic_text = if entry.context.trim().is_empty() && entry.tags.is_empty() {
            entry.source.clone()
        } else {
            let tags = if entry.tags.is_empty() {
                String::new()
            } else {
                format!("\nTags: {}", entry.tags.join(" | "))
            };
            format!(
                "{}\nContext: {}{}",
                entry.source,
                entry.context.trim(),
                tags
            )
        };
        deterministic_ids.push(id.clone());
        semantic_candidates.push(SemanticCandidate {
            id: id.clone(),
            text: semantic_text,
        });
        by_id.insert(id, (entry, hit.score));
    }

    let semantic_result_limit = config
        .max_memory_hits
        .saturating_mul(4)
        .max(config.max_memory_hits)
        .max(1)
        .min(semantic_candidates.len());
    let request =
        SemanticRerankRequest::new(source_text, semantic_candidates, semantic_result_limit);

    let response = match sidecar.rerank(&request) {
        Ok(response) => response,
        Err(error) => {
            status.error = Some(error.to_string());
            return HybridMemoryCandidates {
                candidates: deterministic,
                semantic: status,
            };
        }
    };

    let fused = reciprocal_rank_fusion(
        &deterministic_ids,
        &response.scores,
        ReciprocalRankFusionConfig::default(),
    );
    let max_fused_score = fused
        .first()
        .map(|item| item.score)
        .unwrap_or(1.0)
        .max(f32::EPSILON);

    let mut candidates = deterministic
        .into_iter()
        .filter(|candidate| candidate.kind == ContextKind::Glossary)
        .collect::<Vec<_>>();

    for fused_hit in fused.into_iter().take(config.max_memory_hits) {
        let Some((entry, deterministic_score)) = by_id.get(&fused_hit.id).copied() else {
            continue;
        };
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
        let relevance = (fused_hit.score / max_fused_score).clamp(0.0, 1.0);
        let reason = match (fused_hit.deterministic_rank, fused_hit.semantic_rank) {
            (Some(native_rank), Some(semantic_rank)) => format!(
                "hybrid TM retrieval: native_rank={native_rank}, semantic_rank={semantic_rank}, native_score={deterministic_score:.3}, model={}",
                response.model
            ),
            (Some(native_rank), None) => format!(
                "deterministic TM candidate retained after hybrid fusion: native_rank={native_rank}, native_score={deterministic_score:.3}"
            ),
            (None, Some(semantic_rank)) => format!(
                "semantic TM retrieval: semantic_rank={semantic_rank}, model={}",
                response.model
            ),
            (None, None) => "hybrid TM retrieval".to_string(),
        };
        candidates.push(ContextCandidate::new(
            fused_hit.id,
            ContextKind::TranslationMemory,
            ContextAuthority::Deterministic,
            text,
            reason,
            relevance,
        ));
    }

    status.used = true;
    status.model = Some(response.model);
    HybridMemoryCandidates {
        candidates,
        semantic: status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::GlossaryEntry;
    use crate::MemoryEntry;

    #[test]
    fn no_sidecar_preserves_native_candidates_exactly() {
        let mut memory = TranslationMemory::new();
        memory.add(MemoryEntry::new(
            "his hands were shaking".into(),
            "دست‌هایش می‌لرزید".into(),
            "fear".into(),
        ));
        let mut glossary = Glossary::default();
        glossary.add(GlossaryEntry {
            source_term: "Portal".into(),
            preferred_translation: "پرتال".into(),
            context: "fantasy term".into(),
        });
        let config = MemoryContextConfig::default();
        let expected = native_memory_candidates(
            "His hands were shaking near the Portal.",
            &memory,
            &glossary,
            &config,
        );
        let result = hybrid_memory_candidates(
            "His hands were shaking near the Portal.",
            &memory,
            &glossary,
            &config,
            None,
        );
        assert_eq!(result.candidates, expected);
        assert!(!result.semantic.attempted);
        assert!(!result.semantic.used);
    }

    #[test]
    fn missing_sidecar_falls_back_without_losing_native_memory() {
        let mut memory = TranslationMemory::new();
        memory.add(MemoryEntry::new(
            "I missed you".into(),
            "دلم برات تنگ شده بود".into(),
            "confession".into(),
        ));
        let glossary = Glossary::default();
        let sidecar = SemanticSidecar::new("/definitely/missing/semantic-retrieval-tool");
        let result = hybrid_memory_candidates(
            "I really missed you",
            &memory,
            &glossary,
            &MemoryContextConfig::default(),
            Some(&sidecar),
        );
        assert!(!result.candidates.is_empty());
        assert!(result.semantic.attempted);
        assert!(!result.semantic.used);
        assert!(result.semantic.error.is_some());
    }
}
