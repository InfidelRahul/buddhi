use dhi_core::error::{DhiError, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct VerificationResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl VerificationResult {
    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn compress_errors(&self) -> String {
        if self.success {
            return String::new();
        }
        // Simple error compression: extract lines containing "error:"
        let mut compressed = String::new();
        for line in self.stderr.lines() {
            if line.contains("error:") || line.contains("Error") {
                compressed.push_str(line);
                compressed.push('\n');
            }
        }
        if compressed.is_empty() {
            self.stderr.clone()
        } else {
            compressed
        }
    }
}

pub struct VerifyRunner {
    root: PathBuf,
}

impl VerifyRunner {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn run_cargo_check(&self) -> Result<VerificationResult> {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=short")
            .current_dir(&self.root)
            .output()
            .map_err(|e| DhiError::Config(format!("Failed to run cargo check: {}", e)))?;

        Ok(VerificationResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
