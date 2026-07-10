//! Herculean Desktop - Run AI assistants locally with Docker
//!
//! This is the main Tauri application that manages Docker containers
//! for running Herculean AI assistants locally.

mod docker;
mod container;
mod assistant;
mod error;

use assistant::{Assistant, AssistantManager, AssistantStatus};
use docker::DockerManager;
use container::ContainerManager;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// Application state shared across commands
pub struct AppState {
    pub docker_manager: DockerManager,
    pub container_manager: ContainerManager,
    pub assistant_manager: Mutex<AssistantManager>,
}

/// Docker status response
#[derive(Serialize)]
pub struct DockerStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
}

// ============================================================================
// Docker Management Commands
// ============================================================================

/// Check if Docker is installed and running
#[tauri::command]
async fn check_docker_status(state: State<'_, AppState>) -> Result<DockerStatus, String> {
    let installed = state.docker_manager.is_installed().await;
    let running = if installed {
        state.docker_manager.is_running().await
    } else {
        false
    };
    let version = if installed {
        state.docker_manager.get_version().await.ok()
    } else {
        None
    };

    Ok(DockerStatus {
        installed,
        running,
        version,
    })
}

/// Get Docker Desktop download URL for current platform
#[tauri::command]
fn get_docker_download_url() -> Result<String, String> {
    DockerManager::get_download_url().map_err(|e| e.to_string())
}

/// Start Docker daemon
#[tauri::command]
async fn start_docker(state: State<'_, AppState>) -> Result<(), String> {
    state.docker_manager.start_daemon().await.map_err(|e| e.to_string())
}

/// Wait for Docker to be ready
#[tauri::command]
async fn wait_for_docker(state: State<'_, AppState>) -> Result<(), String> {
    state.docker_manager.wait_for_ready(60).await.map_err(|e| e.to_string())
}

// ============================================================================
// Container Management Commands
// ============================================================================

/// Pull a container image
#[tauri::command]
async fn pull_image(
    state: State<'_, AppState>,
    image: String,
) -> Result<(), String> {
    state.container_manager.pull_image(&image).await.map_err(|e| e.to_string())
}

/// Check if an image exists locally
#[tauri::command]
async fn image_exists(
    state: State<'_, AppState>,
    image: String,
) -> Result<bool, String> {
    state.container_manager.image_exists(&image).await.map_err(|e| e.to_string())
}

/// Start a container for an assistant
#[tauri::command]
async fn start_container(
    state: State<'_, AppState>,
    assistant_id: String,
) -> Result<String, String> {
    // Get assistant data and drop the lock before async operation
    let assistant = {
        let manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;
        manager.get(&assistant_id).ok_or("Assistant not found")?.clone()
    };

    let container_id = state.container_manager
        .start_container(&assistant)
        .await
        .map_err(|e| e.to_string())?;

    // Re-acquire lock to update status
    let mut manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;
    manager.update_status(&assistant_id, AssistantStatus::Running, Some(container_id.clone()))
        .map_err(|e| e.to_string())?;

    Ok(container_id)
}

/// Stop a container
#[tauri::command]
async fn stop_container(
    state: State<'_, AppState>,
    assistant_id: String,
) -> Result<(), String> {
    // Get container_id and drop the lock before async operation
    let container_id = {
        let manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;
        let assistant = manager.get(&assistant_id).ok_or("Assistant not found")?;
        assistant.container_id.clone()
    };

    if let Some(cid) = container_id {
        state.container_manager
            .stop_container(&cid)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Re-acquire lock to update status
    let mut manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;
    manager.update_status(&assistant_id, AssistantStatus::Stopped, None)
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Assistant Management Commands
// ============================================================================

/// Get all assistants
#[tauri::command]
fn get_assistants(state: State<'_, AppState>) -> Result<Vec<Assistant>, String> {
    let manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.list())
}

/// Add a new assistant
#[tauri::command]
async fn add_assistant(
    state: State<'_, AppState>,
    name: String,
    description: String,
    image: String,
    port: u16,
) -> Result<Assistant, String> {
    let mut manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;
    manager.add(name, description, image, port).map_err(|e| e.to_string())
}

/// Remove an assistant
#[tauri::command]
async fn remove_assistant(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Get container_id and drop the lock before async operation
    let container_id = {
        let manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;
        manager.get(&id).and_then(|a| a.container_id.clone())
    };

    // Stop the container if running (after lock is dropped)
    if let Some(cid) = container_id {
        let _ = state.container_manager.stop_container(&cid).await;
    }

    // Re-acquire lock to remove the assistant
    let mut manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;
    manager.remove(&id).map_err(|e| e.to_string())
}

/// Import assistant from download code/URL
#[tauri::command]
async fn import_assistant(
    state: State<'_, AppState>,
    import_code: String,
) -> Result<Assistant, String> {
    // Parse the import code (JSON emitted by the platform's import-code endpoint)
    let assistant_data = parse_import_code(&import_code)?;

    let mut manager = state.assistant_manager.lock().map_err(|e| e.to_string())?;

    // Create the basic assistant
    let mut assistant = manager.add(
        assistant_data.name,
        assistant_data.description,
        assistant_data.image,
        assistant_data.port,
    ).map_err(|e| e.to_string())?;

    // Update with additional fields from import code
    if let Some(env_vars) = assistant_data.env_vars {
        manager.update_env_vars(&assistant.id, env_vars.clone()).map_err(|e| e.to_string())?;
        assistant.env_vars = env_vars;
    }

    // Update vnc_enabled flag
    if let Some(vnc_enabled) = assistant_data.vnc_enabled {
        manager.update_vnc_enabled(&assistant.id, vnc_enabled).map_err(|e| e.to_string())?;
        assistant.vnc_enabled = vnc_enabled;
    }

    // Update instructions if provided (store in env_vars)
    if let Some(instructions) = assistant_data.instructions {
        if !instructions.is_empty() {
            let mut env_vars = assistant.env_vars.clone();
            env_vars.insert("INSTRUCTIONS".to_string(), instructions);
            manager.update_env_vars(&assistant.id, env_vars.clone()).map_err(|e| e.to_string())?;
            assistant.env_vars = env_vars;
        }
    }

    Ok(assistant)
}

#[derive(Deserialize)]
struct ImportData {
    name: String,
    description: String,
    image: String,
    port: u16,
    #[serde(default)]
    env_vars: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    vnc_enabled: Option<bool>,
    #[serde(default)]
    instructions: Option<String>,
}

fn parse_import_code(code: &str) -> Result<ImportData, String> {
    serde_json::from_str::<ImportData>(code)
        .map_err(|_| "Invalid import code format".to_string())
}

// ============================================================================
// Application Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize managers
    let docker_manager = DockerManager::new();
    let container_manager = ContainerManager::new();
    let assistant_manager = AssistantManager::load().unwrap_or_else(|_| AssistantManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            docker_manager,
            container_manager,
            assistant_manager: Mutex::new(assistant_manager),
        })
        .invoke_handler(tauri::generate_handler![
            // Docker commands
            check_docker_status,
            get_docker_download_url,
            start_docker,
            wait_for_docker,
            // Container commands
            pull_image,
            image_exists,
            start_container,
            stop_container,
            // Assistant commands
            get_assistants,
            add_assistant,
            remove_assistant,
            import_assistant,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
