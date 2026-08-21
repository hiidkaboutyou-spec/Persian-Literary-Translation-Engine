use literary_intelligence_engine::models::decision_trace::DecisionTrace;
use uuid::Uuid;

#[test]
fn decision_trace_preserves_explanation_context() {
    let decision_id = Uuid::new_v4();

    let trace = DecisionTrace {
        id: Uuid::new_v4(),
        decision_id,
        context_summary: "Character distrusts the other person after a betrayal".into(),
        influencing_factors: vec![
            "previous chapter conflict".into(),
            "relationship trust is low".into(),
        ],
        confidence: 0.9,
        human_override: None,
    };

    assert!(trace.validate().is_ok());
    assert_eq!(trace.influencing_factors.len(), 2);
}

#[test]
fn decision_trace_rejects_invalid_confidence() {
    let trace = DecisionTrace {
        id: Uuid::new_v4(),
        decision_id: Uuid::new_v4(),
        context_summary: "test".into(),
        influencing_factors: vec![],
        confidence: 1.5,
        human_override: Some("editor changed interpretation".into()),
    };

    assert!(trace.validate().is_err());
}
