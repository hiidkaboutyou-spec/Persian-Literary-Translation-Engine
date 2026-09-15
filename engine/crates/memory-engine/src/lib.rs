pub mod context;
pub mod context_v2;
pub mod glossary;
pub mod models;
pub mod persistence;
pub mod retrieval;
pub mod semantic;

pub use context::{build_memory_context, MemoryContext, MemoryContextConfig};
pub use context_v2::{
    build_context_packet_v2, native_memory_candidates, stable_evidence_id, ContextAuthority,
    ContextCandidate, ContextItem, ContextKind, ContextPacketBudget, ContextPacketConfig,
    ContextPacketV2, CONTEXT_PACKET_SCHEMA_VERSION,
};
pub use models::MemoryEntry;
pub use persistence::{
    load_glossary, load_translation_memory, save_glossary, save_translation_memory,
    PersistenceError,
};
pub use retrieval::{rank_memory, RetrievalConfig, RetrievalHit};
pub use semantic::{
    reciprocal_rank_fusion, FusedRank, ReciprocalRankFusionConfig, SemanticCandidate,
    SemanticError, SemanticMode, SemanticRerankRequest, SemanticRerankResponse, SemanticScore,
    SemanticSidecar, SEMANTIC_PROTOCOL_VERSION,
};

#[derive(Debug, Default, Clone)]
pub struct TranslationMemory {
    entries: Vec<MemoryEntry>,
}

impl TranslationMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, entry: MemoryEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[MemoryEntry] {
        &self.entries
    }

    pub fn search(&self, query: &str) -> Vec<MemoryEntry> {
        self.search_with_config(query, &RetrievalConfig::default())
            .into_iter()
            .map(|hit| hit.entry.clone())
            .collect()
    }

    pub fn search_with_config<'a>(
        &'a self,
        query: &str,
        config: &RetrievalConfig,
    ) -> Vec<RetrievalHit<'a>> {
        rank_memory(&self.entries, query, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_memory_returns_ranked_results() {
        let mut memory = TranslationMemory::new();
        memory.add(MemoryEntry::new(
            "whispered softly".into(),
            "آرام زمزمه کرد".into(),
            "intimate dialogue".into(),
        ));
        memory.add(MemoryEntry::new(
            "shouted across the room".into(),
            "از آن طرف اتاق فریاد زد".into(),
            "argument".into(),
        ));

        let hits = memory.search("softly whispered");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].translation, "آرام زمزمه کرد");
    }
}
