use buddhi_core::error::{DhiError, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileIndexer {
    root: PathBuf,
}

impl FileIndexer {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_dir(&self.root, &mut files)?;
        Ok(files)
    }

    fn walk_dir(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let entries = fs::read_dir(dir).map_err(|e| DhiError::Config(e.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|e| DhiError::Config(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories and target
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "target" {
                        continue;
                    }
                }
                self.walk_dir(&path, files)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
        Ok(())
    }
}
