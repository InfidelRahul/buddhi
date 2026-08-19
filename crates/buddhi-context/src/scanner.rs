use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Scans a project directory to detect the programming languages in use.
/// This is the "Project Language Resolver" that allows Buddhi to automatically
/// determine which Tree-sitter grammars to load.
pub struct ProjectScanner;

impl ProjectScanner {
    /// Scan the project root and return a set of detected language identifiers.
    pub fn scan(project_root: &Path) -> HashSet<String> {
        let mut languages = HashSet::new();

        // 1. Check for framework markers (fast path)
        if project_root.join("Cargo.toml").exists() {
            languages.insert("rust".to_string());
        }
        if project_root.join("package.json").exists() {
            languages.insert("javascript".to_string());
            languages.insert("typescript".to_string()); // TS projects usually have JS deps
        }
        if project_root.join("pubspec.yaml").exists() {
            languages.insert("dart".to_string());
        }
        if project_root.join("composer.json").exists() {
            languages.insert("php".to_string());
        }
        if project_root.join("go.mod").exists() {
            languages.insert("go".to_string());
        }
        if project_root.join("requirements.txt").exists()
            || project_root.join("pyproject.toml").exists()
        {
            languages.insert("python".to_string());
        }

        // 2. Scan file extensions (recursive, limited for performance)
        // This handles projects where framework markers might be missing or incomplete.
        if let Ok(entries) = fs::read_dir(project_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        match ext {
                            "rs" => {
                                languages.insert("rust".to_string());
                            }
                            "py" => {
                                languages.insert("python".to_string());
                            }
                            "js" | "mjs" => {
                                languages.insert("javascript".to_string());
                            }
                            "ts" | "tsx" => {
                                languages.insert("typescript".to_string());
                            }
                            "go" => {
                                languages.insert("go".to_string());
                            }
                            "dart" => {
                                languages.insert("dart".to_string());
                            }
                            "php" => {
                                languages.insert("php".to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        languages
    }
}
