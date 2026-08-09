use dhi_token::budget::TokenBudget;
use dhi_token::counter::CharCounter;
use std::sync::Arc;

#[test]
fn test_budget_enforcement() {
    let counter = Arc::new(CharCounter);
    let mut budget = TokenBudget::new(counter, 10); // 10 tokens ≈ 40 chars

    assert!(budget.check_and_add("Hello world").is_ok());
    assert!(budget
        .check_and_add("This is a very long string that exceeds the budget")
        .is_err());
}

#[test]
fn test_budget_remaining() {
    let counter = Arc::new(CharCounter);
    let mut budget = TokenBudget::new(counter, 100);

    budget.check_and_add("1234567890").unwrap(); // 10 chars ≈ 2.5 tokens
    assert!(budget.remaining() < 100);
}
