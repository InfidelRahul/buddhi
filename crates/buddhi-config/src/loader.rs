use crate::types::Config;
use buddhi_core::error::{DhiError, Result};
use std::fs;
use std::path::Path;

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    if !path.as_ref().exists() {
        tracing::info!("Config file not found, using defaults.");
        return Ok(Config::default());
    }

    let content = fs::read_to_string(path.as_ref()).map_err(DhiError::Io)?;
    let config: Config = serde_yaml::from_str(&content)
        .map_err(|e| DhiError::Config(format!("Failed to parse YAML: {}", e)))?;

    Ok(config)
}
