use dhi_security::path_guard::PathGuard;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_path_traversal_rejection() {
    let root = PathBuf::from("/tmp/test_root");
    // Note: In a real test environment, we would create temp directories.
    // For now, we test the logic with a mock path.
    let result = PathGuard::validate("../etc/passwd", &root);
    assert!(result.is_err());
}
