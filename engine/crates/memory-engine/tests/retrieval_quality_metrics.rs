use std::collections::{HashMap, HashSet};

use memory_engine::{
    evaluate_ranking, MemoryEntry, RetrievalConfig, TranslationMemory,
};

#[test]
fn deterministic_memory_retrieval_has_measurable_ranking_quality() {
    let mut memory = TranslationMemory::new();
    memory.add(MemoryEntry::new(
        "his hands were shaking".into(),
        "دست‌هایش می‌لرزید".into(),
        "fear after a confession".into(),
    ));
    memory.add(MemoryEntry::new(
        "she whispered that she missed him".into(),
        "آرام گفت که دلش برایش تنگ شده بود".into(),
        "quiet confession".into(),
    ));
    memory.add(MemoryEntry::new(
        "sunlight filled the kitchen".into(),
        "نور آفتاب آشپزخانه را پر کرده بود".into(),
        "setting".into(),
    ));

    let query = "He tried to answer after the confession, but his hands were shaking.";
    let hits = memory.search_with_config(
        query,
        &RetrievalConfig {
            max_results: 3,
            min_score: 0.0,
            diversity_threshold: 1.0,
            deduplicate_translations: false,
            ..RetrievalConfig::default()
        },
    );
    let retrieved = hits
        .iter()
        .map(|hit| hit.entry.source.clone())
        .collect::<Vec<_>>();

    let target = "his hands were shaking".to_string();
    let relevant = HashSet::from([target.clone()]);
    let graded = HashMap::from([(target, 3_u32)]);
    let metrics = evaluate_ranking(&retrieved, &relevant, &graded, 3);

    assert_eq!(metrics.reciprocal_rank, 1.0);
    assert_eq!(metrics.recall_at_k, 1.0);
    assert!(metrics.ndcg_at_k > 0.99);
}
