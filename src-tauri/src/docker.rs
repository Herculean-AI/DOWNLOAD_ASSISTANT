//! Docker detection and management module
//!
//! Handles Docker Desktop detection, installation guidance, and daemon management
//! across macOS, Windows, and Linux platforms.

use crate::error::AppError;
use std::process::Command;
use tokio::time::{sleep, Duration};

/// Docker manager for detecting and managing Docker installation
pub struct DockerManager {
    // Reserved for future state
}

impl DockerManager {
    /// Create a new Docker manager
    pub fn new() -> Self {
        Self {}
    }

    /// Check if Docker CLI is installed
    pub async fn is_installed(&self) -> bool {
        let output = Command::new("docker")
            .arg("--version")
            .output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    /// Check if Docker daemon is running
    pub async fn is_running(&self) -> bool {
        let output = Command::new("docker")
            .args(["info"])
            .output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    /// Get Docker version
    pub async fn get_version(&self) -> Result<String, AppError> {
        let output = Command::new("docker")
            .args(["--version"])
            .output()
            .map_err(|e| AppError::Docker(format!("Failed to get Docker version: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Ok(version)
        } else {
            Err(AppError::Docker("Docker version command failed".to_string()))
        }
    }

    /// Get Docker Desktop download URL for the current platform
    pub fn get_download_url() -> Result<String, AppError> {
        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "aarch64")]
            return Ok("https://desktop.docker.com/mac/main/arm64/Docker.dmg".to_string());

            #[cfg(target_arch = "x86_64")]
            return Ok("https://desktop.docker.com/mac/main/amd64/Docker.dmg".to_string());

            #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
            return Err(AppError::Docker("Unsupported macOS architecture".to_string()));
        }

        #[cfg(target_os = "windows")]
        {
            return Ok("https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe".to_string());
        }

        #[cfg(target_os = "linux")]
        {
            // For Linux, we'll provide instructions or package manager commands
            // Docker Desktop for Linux is available but requires specific distros
            return Ok("https://docs.docker.com/desktop/install/linux-install/".to_string());
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            return Err(AppError::Docker("Unsupported operating system".to_string()));
        }
    }

    /// Start Docker daemon/Desktop
    pub async fn start_daemon(&self) -> Result<(), AppError> {
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .args(["-a", "Docker"])
                .output()
                .map_err(|e| AppError::Docker(format!("Failed to start Docker Desktop: {}", e)))?;
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, try to start Docker Desktop
            Command::new("cmd")
                .args(["/C", "start", "", "Docker Desktop"])
                .output()
                .map_err(|e| AppError::Docker(format!("Failed to start Docker Desktop: {}", e)))?;
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, try systemd first, then Docker Desktop
            let systemd_result = Command::new("systemctl")
                .args(["--user", "start", "docker-desktop"])
                .output();

            if systemd_result.is_ok() && systemd_result.unwrap().status.success() {
                return Ok(());
            }

            // Try starting dockerd directly (requires sudo)
            return Err(AppError::Docker(
                "Please start Docker manually: sudo systemctl start docker".to_string()
            ));
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            return Err(AppError::Docker("Unsupported operating system".to_string()));
        }
    }

    /// Wait for Docker to be ready
    pub async fn wait_for_ready(&self, timeout_seconds: u32) -> Result<(), AppError> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_seconds as u64);

        loop {
            if self.is_running().await {
                return Ok(());
            }

            if start.elapsed() > timeout {
                return Err(AppError::Docker(
                    format!("Docker did not become ready within {} seconds", timeout_seconds)
                ));
            }

            sleep(Duration::from_secs(2)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_download_url() {
        let url = DockerManager::get_download_url();
        assert!(url.is_ok());
        let url = url.unwrap();
        assert!(url.starts_with("http"));
    }
}
