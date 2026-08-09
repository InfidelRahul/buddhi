use dhi_core::error::{DhiError, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

pub struct VerifyResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub struct VerifyRunner {
    project_root: PathBuf,
}

impl VerifyRunner {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub async fn run_check(&self) -> Result<VerifyResult> {
        self.run_command("cargo", &["check", "--message-format=short"])
            .await
    }

    pub async fn run_clippy(&self) -> Result<VerifyResult> {
        self.run_command(
            "cargo",
            &["clippy", "--message-format=short", "--", "-D", "warnings"],
        )
        .await
    }

    pub async fn run_tests(&self) -> Result<VerifyResult> {
        self.run_command("cargo", &["test", "--no-fail-fast"]).await
    }

    async fn run_command(&self, program: &str, args: &[&str]) -> Result<VerifyResult> {
        let output = Command::new(program)
            .args(args)
            .current_dir(&self.project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                DhiError::ToolExecution(format!("Failed to execute {}: {}", program, e))
            })?;

        // Use from_utf8_lossy to safely handle compiler output without unwrapping
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(VerifyResult {
            success: output.status.success(),
            stdout,
            stderr,
        })
    }
}
