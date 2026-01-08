import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Dashboard from "./components/Dashboard";
import Chat from "./components/Chat";
import DockerSetup from "./components/DockerSetup";
import ImportAssistant from "./components/ImportAssistant";
import "./App.css";

// Types matching Rust structs
export interface Assistant {
  id: string;
  name: string;
  description: string;
  image: string;
  port: number;
  status: "stopped" | "starting" | "running" | "error";
  container_id: string | null;
  env_vars: Record<string, string>;
  created_at: string;
  last_used: string | null;
  icon: string | null;
  vnc_enabled: boolean;
}

export interface DockerStatus {
  installed: boolean;
  running: boolean;
  version: string | null;
}

type View = "dashboard" | "chat" | "docker-setup" | "import";

function App() {
  const [view, setView] = useState<View>("dashboard");
  const [dockerStatus, setDockerStatus] = useState<DockerStatus | null>(null);
  const [assistants, setAssistants] = useState<Assistant[]>([]);
  const [selectedAssistant, setSelectedAssistant] = useState<Assistant | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Check Docker status on mount
  useEffect(() => {
    checkDockerAndLoad();
  }, []);

  const checkDockerAndLoad = async () => {
    setLoading(true);
    setError(null);
    try {
      const status = await invoke<DockerStatus>("check_docker_status");
      setDockerStatus(status);

      if (status.installed && status.running) {
        await loadAssistants();
        setView("dashboard");
      } else {
        setView("docker-setup");
      }
    } catch (err) {
      console.error("Error checking Docker:", err);
      setError(String(err));
      setView("docker-setup");
    } finally {
      setLoading(false);
    }
  };

  const loadAssistants = async () => {
    try {
      const list = await invoke<Assistant[]>("get_assistants");
      setAssistants(list);
    } catch (err) {
      console.error("Error loading assistants:", err);
    }
  };

  const handleStartAssistant = async (assistant: Assistant) => {
    try {
      // Update local state to starting
      setAssistants(prev => prev.map(a =>
        a.id === assistant.id ? { ...a, status: "starting" as const } : a
      ));

      // Check if image exists, pull if not
      const exists = await invoke<boolean>("image_exists", { image: assistant.image });
      if (!exists) {
        await invoke("pull_image", { image: assistant.image });
      }

      // Start container
      await invoke<string>("start_container", { assistantId: assistant.id });

      // Reload assistants to get updated state
      await loadAssistants();
    } catch (err) {
      console.error("Error starting assistant:", err);
      setError(String(err));
      // Reset status on error
      await loadAssistants();
    }
  };

  const handleStopAssistant = async (assistant: Assistant) => {
    try {
      await invoke("stop_container", { assistantId: assistant.id });
      await loadAssistants();
    } catch (err) {
      console.error("Error stopping assistant:", err);
      setError(String(err));
    }
  };

  const handleOpenChat = (assistant: Assistant) => {
    setSelectedAssistant(assistant);
    setView("chat");
  };

  const handleRemoveAssistant = async (assistant: Assistant) => {
    if (!confirm(`Are you sure you want to remove "${assistant.name}"?`)) {
      return;
    }
    try {
      await invoke("remove_assistant", { id: assistant.id });
      await loadAssistants();
    } catch (err) {
      console.error("Error removing assistant:", err);
      setError(String(err));
    }
  };

  const handleImportComplete = async () => {
    await loadAssistants();
    setView("dashboard");
  };

  const handleDockerReady = async () => {
    await checkDockerAndLoad();
  };

  if (loading) {
    return (
      <div className="app loading-screen">
        <div className="loading-content">
          <div className="spinner"></div>
          <h2>Herculean Desktop</h2>
          <p>Checking system requirements...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}

      {view === "docker-setup" && (
        <DockerSetup
          dockerStatus={dockerStatus}
          onReady={handleDockerReady}
        />
      )}

      {view === "dashboard" && (
        <Dashboard
          assistants={assistants}
          onStartAssistant={handleStartAssistant}
          onStopAssistant={handleStopAssistant}
          onOpenChat={handleOpenChat}
          onRemoveAssistant={handleRemoveAssistant}
          onImportClick={() => setView("import")}
          onRefresh={loadAssistants}
        />
      )}

      {view === "chat" && selectedAssistant && (
        <Chat
          assistant={selectedAssistant}
          onBack={() => {
            setSelectedAssistant(null);
            setView("dashboard");
          }}
        />
      )}

      {view === "import" && (
        <ImportAssistant
          onComplete={handleImportComplete}
          onCancel={() => setView("dashboard")}
        />
      )}
    </div>
  );
}

export default App;
