//! Container lifecycle management module
//!
//! Handles Docker container operations: pull, run, stop, logs, etc.

use crate::assistant::Assistant;
use crate::error::AppError;
use std::process::Command;

/// Container manager for Docker container operations
pub struct ContainerManager {
    // Reserved for future state (e.g., connection pool)
}

impl ContainerManager {
    /// Create a new container manager
    pub fn new() -> Self {
        Self {}
    }

    /// Pull a Docker image
    pub async fn pull_image(&self, image: &str) -> Result<(), AppError> {
        let output = Command::new("docker")
            .args(["pull", image])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to pull image: {}", e)))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Container(format!("Failed to pull image {}: {}", image, stderr)))
        }
    }

    /// Check if an image exists locally
    pub async fn image_exists(&self, image: &str) -> Result<bool, AppError> {
        let output = Command::new("docker")
            .args(["image", "inspect", image])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to check image: {}", e)))?;

        Ok(output.status.success())
    }

    /// Start a container for an assistant
    pub async fn start_container(&self, assistant: &Assistant) -> Result<String, AppError> {
        // Generate a unique container name
        let container_name = format!("herculean-{}", assistant.id);

        // Check if container already exists and remove it
        let _ = self.remove_container(&container_name).await;

        // Build the docker run command
        let mut args = vec![
            "run".to_string(),
            "-d".to_string(),
            "--name".to_string(),
            container_name.clone(),
            "-p".to_string(),
            format!("{}:8080", assistant.port),
        ];

        // Add environment variables if present
        for (key, value) in &assistant.env_vars {
            args.push("-e".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Add the image
        args.push(assistant.image.clone());

        let output = Command::new("docker")
            .args(&args)
            .output()
            .map_err(|e| AppError::Container(format!("Failed to start container: {}", e)))?;

        if output.status.success() {
            let container_id = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Ok(container_id)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Container(format!("Failed to start container: {}", stderr)))
        }
    }

    /// Stop a container
    pub async fn stop_container(&self, container_id: &str) -> Result<(), AppError> {
        let output = Command::new("docker")
            .args(["stop", container_id])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to stop container: {}", e)))?;

        if output.status.success() {
            // Also remove the container
            let _ = self.remove_container(container_id).await;
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Container(format!("Failed to stop container: {}", stderr)))
        }
    }

    /// Remove a container
    pub async fn remove_container(&self, container_id: &str) -> Result<(), AppError> {
        let output = Command::new("docker")
            .args(["rm", "-f", container_id])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to remove container: {}", e)))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Container(format!("Failed to remove container: {}", stderr)))
        }
    }

    /// Get container logs
    pub async fn get_logs(&self, container_id: &str, lines: u32) -> Result<String, AppError> {
        let output = Command::new("docker")
            .args(["logs", "--tail", &lines.to_string(), container_id])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to get logs: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(format!("{}{}", stdout, stderr))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Container(format!("Failed to get logs: {}", stderr)))
        }
    }

    /// Check if a container is running
    pub async fn is_running(&self, container_id: &str) -> Result<bool, AppError> {
        let output = Command::new("docker")
            .args(["inspect", "-f", "{{.State.Running}}", container_id])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to check container status: {}", e)))?;

        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            Ok(status == "true")
        } else {
            Ok(false)
        }
    }

    /// Get container health status
    pub async fn get_health_status(&self, container_id: &str) -> Result<String, AppError> {
        let output = Command::new("docker")
            .args(["inspect", "-f", "{{.State.Health.Status}}", container_id])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to get health status: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    /// List all Herculean containers
    pub async fn list_herculean_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        let output = Command::new("docker")
            .args([
                "ps",
                "-a",
                "--filter", "name=herculean-",
                "--format", "{{.ID}}|{{.Names}}|{{.Status}}|{{.Ports}}"
            ])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to list containers: {}", e)))?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let containers: Vec<ContainerInfo> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                ContainerInfo {
                    id: parts.get(0).unwrap_or(&"").to_string(),
                    name: parts.get(1).unwrap_or(&"").to_string(),
                    status: parts.get(2).unwrap_or(&"").to_string(),
                    ports: parts.get(3).unwrap_or(&"").to_string(),
                }
            })
            .collect();

        Ok(containers)
    }

    /// Remove an image
    pub async fn remove_image(&self, image: &str) -> Result<(), AppError> {
        let output = Command::new("docker")
            .args(["rmi", "-f", image])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to remove image: {}", e)))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Container(format!("Failed to remove image: {}", stderr)))
        }
    }

    /// Get image size
    pub async fn get_image_size(&self, image: &str) -> Result<String, AppError> {
        let output = Command::new("docker")
            .args(["image", "inspect", "-f", "{{.Size}}", image])
            .output()
            .map_err(|e| AppError::Container(format!("Failed to get image size: {}", e)))?;

        if output.status.success() {
            let size_bytes: i64 = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .unwrap_or(0);

            // Convert to human-readable format
            let size = if size_bytes >= 1_000_000_000 {
                format!("{:.2} GB", size_bytes as f64 / 1_000_000_000.0)
            } else if size_bytes >= 1_000_000 {
                format!("{:.2} MB", size_bytes as f64 / 1_000_000.0)
            } else {
                format!("{} KB", size_bytes / 1000)
            };

            Ok(size)
        } else {
            Err(AppError::Container("Failed to get image size".to_string()))
        }
    }
}

/// Container information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub ports: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_container_manager_creation() {
        let manager = ContainerManager::new();
        // Just verify it creates without error
        assert!(true);
    }
}
