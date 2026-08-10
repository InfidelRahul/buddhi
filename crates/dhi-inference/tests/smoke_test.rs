use dhi_inference::context::ContextManager;

#[test]
fn test_context_manager_truncation() {
    let mut manager = ContextManager::new(10); // 10 tokens ≈ 40 chars

    manager
        .add_message("This is a test message that is long enough")
        .unwrap();
    manager.add_message("Short").unwrap();

    // Verify context was truncated to fit within 10 tokens
    let context = manager.get_context();
    assert!(context.len() <= 40);
}
