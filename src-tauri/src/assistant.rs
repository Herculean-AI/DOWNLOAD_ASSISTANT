//! Assistant data management module
//!
//! Handles storage and retrieval of assistant configurations on disk.

use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Assistant status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssistantStatus {
    Stopped,
    Starting,
    Running,
    Error,
}

impl Default for AssistantStatus {
    fn default() -> Self {
        AssistantStatus::Stopped
    }
}

/// Assistant configuration and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assistant {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Description of what this assistant does
    pub description: String,

    /// Docker image to use
    pub image: String,

    /// Local port to expose the assistant on
    pub port: u16,

    /// Current status
    pub status: AssistantStatus,

    /// Docker container ID when running
    #[serde(default)]
    pub container_id: Option<String>,

    /// Environment variables for the container
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    /// When the assistant was added
    pub created_at: DateTime<Utc>,

    /// Last time the assistant was used
    #[serde(default)]
    pub last_used: Option<DateTime<Utc>>,

    /// Icon URL or emoji
    #[serde(default)]
    pub icon: Option<String>,

    /// Whether this assistant uses VNC for browser viewing
    #[serde(default)]
    pub vnc_enabled: bool,
}

impl Assistant {
    /// Create a new assistant
    pub fn new(name: String, description: String, image: String, port: u16) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            image,
            port,
            status: AssistantStatus::Stopped,
            container_id: None,
            env_vars: HashMap::new(),
            created_at: Utc::now(),
            last_used: None,
            icon: None,
            vnc_enabled: false,
        }
    }

    /// Get the local URL for this assistant
    pub fn get_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Get the WebSocket URL for this assistant
    pub fn get_ws_url(&self) -> String {
        format!("ws://localhost:{}/ws", self.port)
    }

    /// Get the VNC URL if VNC is enabled
    pub fn get_vnc_url(&self) -> Option<String> {
        if self.vnc_enabled {
            Some(format!("http://localhost:{}/novnc/vnc.html", self.port))
        } else {
            None
        }
    }
}

/// Storage for assistants
#[derive(Debug, Serialize, Deserialize)]
pub struct AssistantStorage {
    pub assistants: Vec<Assistant>,
    pub version: u32,
}

impl Default for AssistantStorage {
    fn default() -> Self {
        Self {
            assistants: vec![],
            version: 1,
        }
    }
}

/// Manager for assistant CRUD operations
pub struct AssistantManager {
    storage: AssistantStorage,
    storage_path: PathBuf,
    next_port: u16,
}

impl AssistantManager {
    /// Create a new assistant manager
    pub fn new() -> Self {
        let storage_path = Self::get_storage_path();
        Self {
            storage: AssistantStorage::default(),
            storage_path,
            next_port: 8081, // Start from 8081, leaving 8080 for potential conflicts
        }
    }

    /// Load assistant manager from disk
    pub fn load() -> Result<Self, AppError> {
        let storage_path = Self::get_storage_path();

        // Ensure directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Storage(format!("Failed to create storage directory: {}", e)))?;
        }

        // Load or create storage
        let storage = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path)
                .map_err(|e| AppError::Storage(format!("Failed to read storage file: {}", e)))?;
            serde_json::from_str(&content)
                .map_err(|e| AppError::Storage(format!("Failed to parse storage file: {}", e)))?
        } else {
            AssistantStorage::default()
        };

        // Calculate next available port
        let next_port = storage.assistants
            .iter()
            .map(|a| a.port)
            .max()
            .map(|p| p + 1)
            .unwrap_or(8081);

        Ok(Self {
            storage,
            storage_path,
            next_port,
        })
    }

    /// Get the storage file path
    fn get_storage_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("herculean-desktop")
            .join("assistants.json")
    }

    /// Save storage to disk
    fn save(&self) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(&self.storage)
            .map_err(|e| AppError::Storage(format!("Failed to serialize storage: {}", e)))?;

        // Ensure directory exists
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Storage(format!("Failed to create storage directory: {}", e)))?;
        }

        fs::write(&self.storage_path, content)
            .map_err(|e| AppError::Storage(format!("Failed to write storage file: {}", e)))?;

        Ok(())
    }

    /// List all assistants
    pub fn list(&self) -> Vec<Assistant> {
        self.storage.assistants.clone()
    }

    /// Get an assistant by ID
    pub fn get(&self, id: &str) -> Option<&Assistant> {
        self.storage.assistants.iter().find(|a| a.id == id)
    }

    /// Get a mutable reference to an assistant by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Assistant> {
        self.storage.assistants.iter_mut().find(|a| a.id == id)
    }

    /// Add a new assistant
    pub fn add(
        &mut self,
        name: String,
        description: String,
        image: String,
        port: u16,
    ) -> Result<Assistant, AppError> {
        // Use provided port or allocate next available
        let actual_port = if port > 0 {
            port
        } else {
            let p = self.next_port;
            self.next_port += 1;
            p
        };

        // Check for port conflicts
        if self.storage.assistants.iter().any(|a| a.port == actual_port) {
            return Err(AppError::Storage(format!("Port {} is already in use by another assistant", actual_port)));
        }

        let assistant = Assistant::new(name, description, image, actual_port);
        self.storage.assistants.push(assistant.clone());
        self.save()?;

        Ok(assistant)
    }

    /// Remove an assistant
    pub fn remove(&mut self, id: &str) -> Result<(), AppError> {
        let initial_len = self.storage.assistants.len();
        self.storage.assistants.retain(|a| a.id != id);

        if self.storage.assistants.len() == initial_len {
            return Err(AppError::Storage(format!("Assistant {} not found", id)));
        }

        self.save()?;
        Ok(())
    }

    /// Update assistant status
    pub fn update_status(
        &mut self,
        id: &str,
        status: AssistantStatus,
        container_id: Option<String>,
    ) -> Result<(), AppError> {
        if let Some(assistant) = self.get_mut(id) {
            assistant.status = status;
            assistant.container_id = container_id;
            if assistant.status == AssistantStatus::Running {
                assistant.last_used = Some(Utc::now());
            }
            self.save()?;
            Ok(())
        } else {
            Err(AppError::Storage(format!("Assistant {} not found", id)))
        }
    }

    /// Update assistant environment variables
    pub fn update_env_vars(
        &mut self,
        id: &str,
        env_vars: HashMap<String, String>,
    ) -> Result<(), AppError> {
        if let Some(assistant) = self.get_mut(id) {
            assistant.env_vars = env_vars;
            self.save()?;
            Ok(())
        } else {
            Err(AppError::Storage(format!("Assistant {} not found", id)))
        }
    }

    /// Check if any assistants exist
    pub fn is_empty(&self) -> bool {
        self.storage.assistants.is_empty()
    }

    /// Get count of assistants
    pub fn count(&self) -> usize {
        self.storage.assistants.len()
    }

    /// Find assistants by status
    pub fn find_by_status(&self, status: AssistantStatus) -> Vec<&Assistant> {
        self.storage.assistants
            .iter()
            .filter(|a| a.status == status)
            .collect()
    }

    /// Reset all assistant statuses to Stopped (useful on app startup)
    pub fn reset_all_statuses(&mut self) -> Result<(), AppError> {
        for assistant in &mut self.storage.assistants {
            assistant.status = AssistantStatus::Stopped;
            assistant.container_id = None;
        }
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assistant_creation() {
        let assistant = Assistant::new(
            "Test Assistant".to_string(),
            "A test assistant".to_string(),
            "test:latest".to_string(),
            8080,
        );

        assert!(!assistant.id.is_empty());
        assert_eq!(assistant.name, "Test Assistant");
        assert_eq!(assistant.status, AssistantStatus::Stopped);
    }

    #[test]
    fn test_assistant_urls() {
        let assistant = Assistant::new(
            "Test".to_string(),
            "Test".to_string(),
            "test:latest".to_string(),
            8080,
        );

        assert_eq!(assistant.get_url(), "http://localhost:8080");
        assert_eq!(assistant.get_ws_url(), "ws://localhost:8080/ws");
        assert!(assistant.get_vnc_url().is_none());
    }
}
