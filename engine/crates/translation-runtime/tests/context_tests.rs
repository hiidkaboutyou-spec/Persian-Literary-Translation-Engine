use translation_runtime::context::TranslationContext;

#[test]
fn context_keeps_translation_inputs_separate_from_literary_ownership() {
    let context = TranslationContext::new("hello world".into());

    assert_eq!(context.source_text, "hello world");
    assert!(context.character_context.is_empty());
    assert!(context.previous_decisions.is_empty());
}

#[test]
fn context_can_carry_previous_understanding() {
    let mut context = TranslationContext::new("dialogue".into());
    context.character_context.push("cold speaking style".into());
    context.previous_decisions.push("keep formal distance".into());

    assert_eq!(context.character_context.len(), 1);
    assert_eq!(context.previous_decisions.len(), 1);
}
