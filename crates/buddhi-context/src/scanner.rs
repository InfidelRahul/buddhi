use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Scans a project directory to detect the programming languages in use.
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
            languages.insert("typescript".to_string());
            languages.insert("javascript".to_string());
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
        if project_root.join("build.gradle").exists() || project_root.join("pom.xml").exists() {
            languages.insert("java".to_string());
        }

        // 2. Scan file extensions and verify against the language pack
        if let Ok(entries) = fs::read_dir(project_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        // Map common extensions to language pack names
                        let lang = match ext {
                            "rs" => Some("rust"),
                            "py" => Some("python"),
                            "js" | "mjs" | "cjs" => Some("javascript"),
                            "ts" | "tsx" => Some("typescript"),
                            "go" => Some("go"),
                            "dart" => Some("dart"),
                            "php" => Some("php"),
                            "java" => Some("java"),
                            "rb" => Some("ruby"),
                            "c" | "h" => Some("c"),
                            "cpp" | "cc" | "hpp" => Some("cpp"),
                            "cs" => Some("c_sharp"),
                            "swift" => Some("swift"),
                            "kt" | "kts" => Some("kotlin"),
                            "scala" => Some("scala"),
                            "sh" | "bash" => Some("bash"),
                            "yaml" | "yml" => Some("yaml"),
                            "toml" => Some("toml"),
                            "json" => Some("json"),
                            "sql" => Some("sql"),
                            _ => None,
                        };
                        if let Some(l) = lang {
                            // Verify the pack actually supports this language
                            if tree_sitter_language_pack::has_language(l) {
                                languages.insert(l.to_string());
                            }
                        }
                    }
                }
            }
        }

        languages
    }
}
