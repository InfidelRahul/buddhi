use dhi_security::path_guard::PathGuard;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_valid_path_inside_root() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file_path = root.join("test.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    let result = PathGuard::validate("test.rs", root);
    assert!(result.is_ok(), "Valid file inside root should be allowed");
}

#[test]
fn test_path_traversal_rejection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Attempt to traverse outside the root
    let result = PathGuard::validate("../outside.txt", root);
    assert!(result.is_err(), "Path traversal should be rejected");
}

#[test]
fn test_hidden_file_rejection() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let hidden_file = root.join(".env");
    fs::write(&hidden_file, "SECRET=123").unwrap();

    let result = PathGuard::validate(".env", root);
    assert!(result.is_err(), "Hidden files like .env should be rejected");
}

#[test]
fn test_absolute_path_inside_root() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file_path = root.join("abs_test.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    // Pass the absolute path of the file
    let result = PathGuard::validate(file_path.to_str().unwrap(), root);
    assert!(
        result.is_ok(),
        "Absolute path inside root should be allowed"
    );
}
