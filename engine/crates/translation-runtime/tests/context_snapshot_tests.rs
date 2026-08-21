use translation_runtime::context_snapshot::ContextSnapshot;

#[test]
fn snapshot_preserves_previous_decisions() {
    let mut snapshot = ContextSnapshot::new("chapter-10".into());
    snapshot.add_decision("preserve distant character voice".into());

    assert_eq!(snapshot.chapter_id, "chapter-10");
    assert_eq!(snapshot.previous_decisions.len(), 1);
}

#[test]
fn snapshot_starts_without_state() {
    let snapshot = ContextSnapshot::new("chapter-1".into());

    assert!(snapshot.character_state.is_empty());
    assert!(snapshot.relationship_state.is_empty());
}
