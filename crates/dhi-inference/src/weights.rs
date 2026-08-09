use dhi_core::error::{DhiError, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

pub struct ModelWeights {
    _file: File,
    mmap: Mmap,
}

impl ModelWeights {
    pub fn load(model_path: &Path) -> Result<Self> {
        let file = File::open(model_path)
            .map_err(|e| DhiError::Config(format!("Failed to open model file: {}", e)))?;

        // Safely memory-map weights to avoid RAM bloat (zero-copy loading)
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| DhiError::Config(format!("Failed to memory-map weights: {}", e)))?;

        Ok(Self { _file: file, mmap })
    }

    pub fn data(&self) -> &[u8] {
        &self.mmap
    }
}
