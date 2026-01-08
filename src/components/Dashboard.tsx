import { Bot, Play, Square, MessageCircle, Trash2, Plus, RefreshCw } from "lucide-react";
import type { Assistant } from "../App";
import "./Dashboard.css";

interface DashboardProps {
  assistants: Assistant[];
  onStartAssistant: (assistant: Assistant) => void;
  onStopAssistant: (assistant: Assistant) => void;
  onOpenChat: (assistant: Assistant) => void;
  onRemoveAssistant: (assistant: Assistant) => void;
  onImportClick: () => void;
  onRefresh: () => void;
}

function Dashboard({
  assistants,
  onStartAssistant,
  onStopAssistant,
  onOpenChat,
  onRemoveAssistant,
  onImportClick,
  onRefresh,
}: DashboardProps) {
  const getStatusColor = (status: Assistant["status"]) => {
    switch (status) {
      case "running": return "#22c55e";
      case "starting": return "#eab308";
      case "error": return "#ef4444";
      default: return "#6b7280";
    }
  };

  const getStatusText = (status: Assistant["status"]) => {
    switch (status) {
      case "running": return "Running";
      case "starting": return "Starting...";
      case "error": return "Error";
      default: return "Stopped";
    }
  };

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <div className="header-left">
          <Bot size={32} className="logo-icon" />
          <h1>Herculean Desktop</h1>
        </div>
        <div className="header-actions">
          <button className="btn btn-secondary" onClick={onRefresh}>
            <RefreshCw size={16} />
            Refresh
          </button>
          <button className="btn btn-primary" onClick={onImportClick}>
            <Plus size={16} />
            Add Assistant
          </button>
        </div>
      </header>

      <main className="dashboard-content">
        {assistants.length === 0 ? (
          <div className="empty-state">
            <Bot size={64} className="empty-icon" />
            <h2>No Assistants Yet</h2>
            <p>
              Download assistants from the Herculean AI platform or add one manually
              to get started.
            </p>
            <button className="btn btn-primary" onClick={onImportClick}>
              <Plus size={16} />
              Add Your First Assistant
            </button>
          </div>
        ) : (
          <div className="assistants-grid">
            {assistants.map((assistant) => (
              <div key={assistant.id} className="assistant-card">
                <div className="assistant-header">
                  <div className="assistant-icon">
                    {assistant.icon || "🤖"}
                  </div>
                  <div className="assistant-info">
                    <h3>{assistant.name}</h3>
                    <p className="assistant-description">{assistant.description}</p>
                  </div>
                </div>

                <div className="assistant-status">
                  <span
                    className="status-indicator"
                    style={{ backgroundColor: getStatusColor(assistant.status) }}
                  />
                  <span className="status-text">{getStatusText(assistant.status)}</span>
                  {assistant.status === "running" && (
                    <span className="port-info">Port {assistant.port}</span>
                  )}
                </div>

                <div className="assistant-actions">
                  {assistant.status === "stopped" && (
                    <button
                      className="btn btn-success"
                      onClick={() => onStartAssistant(assistant)}
                    >
                      <Play size={16} />
                      Start
                    </button>
                  )}

                  {assistant.status === "starting" && (
                    <button className="btn btn-secondary" disabled>
                      <div className="spinner-small" />
                      Starting...
                    </button>
                  )}

                  {assistant.status === "running" && (
                    <>
                      <button
                        className="btn btn-primary"
                        onClick={() => onOpenChat(assistant)}
                      >
                        <MessageCircle size={16} />
                        Open Chat
                      </button>
                      <button
                        className="btn btn-secondary"
                        onClick={() => onStopAssistant(assistant)}
                      >
                        <Square size={16} />
                        Stop
                      </button>
                    </>
                  )}

                  <button
                    className="btn btn-danger btn-icon"
                    onClick={() => onRemoveAssistant(assistant)}
                    title="Remove Assistant"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </main>

      <footer className="dashboard-footer">
        <p>Herculean Desktop v0.1.0</p>
      </footer>
    </div>
  );
}

export default Dashboard;
