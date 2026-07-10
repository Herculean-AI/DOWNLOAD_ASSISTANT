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
