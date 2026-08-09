use dhi_token::budget::TokenBudget;
use dhi_token::counter::CharCounter;
use std::sync::Arc;

#[test]
fn test_budget_enforcement() {
    let counter = Arc::new(CharCounter);
    // Set budget high enough to accommodate "Hello world" regardless of char-to-token ratio
    let mut budget = TokenBudget::new(counter, 100);

    assert!(budget.check_and_add("Hello world").is_ok());

    // Create a string guaranteed to exceed 100 tokens/chars
    let long_string = "a".repeat(500);
    assert!(budget.check_and_add(&long_string).is_err());
}

#[test]
fn test_budget_remaining() {
    let counter = Arc::new(CharCounter);
    let mut budget = TokenBudget::new(counter, 100);

    budget.check_and_add("1234567890").unwrap();
    assert!(budget.remaining() < 100);
}
