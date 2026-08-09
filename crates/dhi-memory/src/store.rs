use chrono::{DateTime, Utc};
use dhi_core::error::{DhiError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub context: String,
    pub lesson: String,
    pub created_at: DateTime<Utc>,
}

pub struct MemoryStore {
    path: PathBuf,
    entries: Vec<MemoryEntry>,
}

impl MemoryStore {
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = project_root.join(".dhi").join("memory.json");
        let entries = if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| DhiError::Config(format!("Failed to read memory: {}", e)))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Self { path, entries })
    }

    pub fn add_lesson(&mut self, context: String, lesson: String) -> Result<()> {
        let entry = MemoryEntry {
            id: Uuid::new_v4().to_string(),
            context,
            lesson,
            created_at: Utc::now(),
        };
        self.entries.push(entry);
        self.save()
    }

    pub fn get_relevant(&self, _query: &str, limit: usize) -> Vec<MemoryEntry> {
        // Placeholder: simple reverse chronological retrieval
        // Future: implement vector similarity or keyword matching
        self.entries.iter().rev().take(limit).cloned().collect()
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(DhiError::Io)?;
        }
        let content =
            serde_json::to_string_pretty(&self.entries).map_err(DhiError::Serialization)?;
        fs::write(&self.path, content).map_err(DhiError::Io)?;
        Ok(())
    }
}
