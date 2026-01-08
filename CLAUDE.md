# CLAUDE.md - Herculean Desktop Application

## Overview

**Herculean Desktop** is a cross-platform desktop application that enables users to download and run Herculean AI assistants locally using Docker. It provides a native desktop experience with automatic Docker management, container lifecycle control, and an integrated chat interface.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Herculean Desktop App                     │
├─────────────────────────────────────────────────────────────┤
│  Frontend (React + TypeScript)                               │
│  ├── Dashboard - Lists all downloaded assistants            │
│  ├── Chat - WebSocket-based conversation interface          │
│  ├── DockerSetup - Docker installation/startup wizard       │
│  └── ImportAssistant - Add assistants via code or manual    │
├─────────────────────────────────────────────────────────────┤
│  Backend (Rust + Tauri)                                      │
│  ├── Docker Manager - Detection, installation, startup      │
│  ├── Container Manager - Pull, run, stop, logs              │
│  └── Assistant Manager - Local storage and state            │
├─────────────────────────────────────────────────────────────┤
│  Docker                                                      │
│  └── Herculean Assistant Containers (from Cloud Run images) │
└─────────────────────────────────────────────────────────────┘
```

## Technology Stack

- **Frontend**: React 19, TypeScript, Vite
- **Backend**: Rust, Tauri 2
- **Styling**: Custom CSS with CSS variables
- **Icons**: Lucide React
- **Container Runtime**: Docker Desktop

## Project Structure

```
DOWNLOAD_ASSISTANT/
├── src/                          # React frontend
│   ├── App.tsx                   # Main application component
│   ├── App.css                   # Global styles
│   └── components/
│       ├── Dashboard.tsx         # Assistant list and controls
│       ├── Dashboard.css
│       ├── Chat.tsx              # WebSocket chat interface
│       ├── Chat.css
│       ├── DockerSetup.tsx       # Docker installation wizard
│       ├── DockerSetup.css
│       ├── ImportAssistant.tsx   # Add new assistants
│       └── ImportAssistant.css
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Tauri commands and app entry
│   │   ├── docker.rs             # Docker detection and management
│   │   ├── container.rs          # Container lifecycle operations
│   │   ├── assistant.rs          # Assistant data storage
│   │   └── error.rs              # Error types
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
│
├── package.json                  # Node.js dependencies
├── vite.config.ts                # Vite configuration
└── CLAUDE.md                     # This file
```

## Key Features

### 1. Docker Auto-Detection and Installation

On first launch, the app:
1. Checks if Docker is installed (`docker --version`)
2. Checks if Docker daemon is running (`docker info`)
3. If not installed, provides download link for Docker Desktop
4. If installed but not running, starts Docker Desktop

### 2. Assistant Management

- **Import via Code**: Paste JSON from web platform
- **Manual Add**: Specify name, image, and port
- **Auto Port Assignment**: Avoids conflicts
- **Persistent Storage**: Saves to `~/Library/Application Support/herculean-desktop/` (macOS)

### 3. Container Lifecycle

- **Pull Images**: Downloads from container registry
- **Start/Stop**: Manages container lifecycle
- **Status Tracking**: Running, Stopped, Starting, Error
- **Port Mapping**: Maps container 8080 to local port

### 4. Chat Interface

- **WebSocket Connection**: Connects to `ws://localhost:{port}/ws`
- **Real-time Messaging**: Streaming responses
- **VNC Support**: Optional browser viewing for browser agents

## Tauri Commands (Rust → Frontend)

### Docker Commands
- `check_docker_status()` → `DockerStatus`
- `get_docker_download_url()` → `String`
- `start_docker()` → `()`
- `wait_for_docker()` → `()`

### Container Commands
- `pull_image(image: String)` → `()`
- `image_exists(image: String)` → `bool`
- `start_container(assistant_id: String)` → `String`
- `stop_container(assistant_id: String)` → `()`
- `get_container_logs(container_id: String, lines: u32)` → `String`

### Assistant Commands
- `get_assistants()` → `Vec<Assistant>`
- `get_assistant(id: String)` → `Option<Assistant>`
- `add_assistant(name, description, image, port)` → `Assistant`
- `remove_assistant(id: String)` → `()`
- `import_assistant(import_code: String)` → `Assistant`

## Data Models

### Assistant
```rust
struct Assistant {
    id: String,
    name: String,
    description: String,
    image: String,
    port: u16,
    status: AssistantStatus,
    container_id: Option<String>,
    env_vars: HashMap<String, String>,
    created_at: DateTime<Utc>,
    last_used: Option<DateTime<Utc>>,
    icon: Option<String>,
    vnc_enabled: bool,
}
```

### Import Code Format
```json
{
  "name": "Email Assistant",
  "description": "Manage your emails with AI",
  "image": "europe-west4-docker.pkg.dev/hercules-core-platform/herculeanai/assistant:latest",
  "port": 8081
}
```

## Development Commands

```bash
# Install dependencies
npm install

# Start development (frontend + Tauri)
npm run tauri dev

# Build for production
npm run tauri build

# Build frontend only
npm run build

# Run frontend dev server only
npm run dev
```

## Build Outputs

| Platform | Output | Location |
|----------|--------|----------|
| macOS | `.app` in `.dmg` | `src-tauri/target/release/bundle/dmg/` |
| Windows | `.exe` installer | `src-tauri/target/release/bundle/nsis/` |
| Linux | `.AppImage`, `.deb` | `src-tauri/target/release/bundle/appimage/` |

## Integration with Herculean Platform

### Web Platform Integration

The web platform (staging.herculean.ai) generates import codes when users click "Download Assistant". This code contains the Docker image reference and configuration.

### Container Images

Assistant containers are the same images deployed to Cloud Run:
- Registry: `europe-west4-docker.pkg.dev/hercules-core-platform/herculeanai/`
- Images follow the same architecture as cloud deployments
- Expose port 8080 with WebSocket at `/ws`

## Security Considerations

- Docker commands run with user privileges (no sudo)
- Container network is isolated to localhost
- No data leaves the machine (except Docker pull)
- Credentials managed by the container internally

## Future Enhancements

1. **Offline Mode**: Bundle container images with the app
2. **Auto-Updates**: Check for app and container updates
3. **Multi-User**: Support multiple user profiles
4. **Sync**: Sync assistant list with cloud account

---

**Last Updated**: 2026-01-07
**Version**: 0.1.0
