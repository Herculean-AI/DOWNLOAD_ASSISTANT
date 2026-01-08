import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, Plus, Download, Loader2 } from "lucide-react";
import type { Assistant } from "../App";
import "./ImportAssistant.css";

interface ImportAssistantProps {
  onComplete: () => void;
  onCancel: () => void;
}

type ImportMode = "code" | "manual";

function ImportAssistant({ onComplete, onCancel }: ImportAssistantProps) {
  const [mode, setMode] = useState<ImportMode>("code");
  const [importCode, setImportCode] = useState("");
  const [manualForm, setManualForm] = useState({
    name: "",
    description: "",
    image: "",
    port: 0,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isPulling, setIsPulling] = useState(false);
  const [pullProgress, setPullProgress] = useState("");

  const handleImportCode = async () => {
    if (!importCode.trim()) {
      setError("Please enter an import code");
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const assistant = await invoke<Assistant>("import_assistant", {
        importCode: importCode.trim(),
      });

      // Pull the image
      setIsPulling(true);
      setPullProgress(`Downloading ${assistant.image}...`);
      await invoke("pull_image", { image: assistant.image });

      onComplete();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
      setIsPulling(false);
    }
  };

  const handleManualAdd = async () => {
    if (!manualForm.name.trim() || !manualForm.image.trim()) {
      setError("Name and Image are required");
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      await invoke<Assistant>("add_assistant", {
        name: manualForm.name.trim(),
        description: manualForm.description.trim() || "Custom assistant",
        image: manualForm.image.trim(),
        port: manualForm.port || 0, // 0 means auto-assign
      });

      // Pull the image
      setIsPulling(true);
      setPullProgress(`Downloading ${manualForm.image}...`);
      await invoke("pull_image", { image: manualForm.image.trim() });

      onComplete();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
      setIsPulling(false);
    }
  };

  return (
    <div className="import-assistant">
      <header className="import-header">
        <button className="btn btn-icon" onClick={onCancel}>
          <ArrowLeft size={20} />
        </button>
        <h1>Add Assistant</h1>
      </header>

      <main className="import-content">
        <div className="mode-tabs">
          <button
            className={`tab ${mode === "code" ? "active" : ""}`}
            onClick={() => setMode("code")}
          >
            <Download size={16} />
            Import from Code
          </button>
          <button
            className={`tab ${mode === "manual" ? "active" : ""}`}
            onClick={() => setMode("manual")}
          >
            <Plus size={16} />
            Add Manually
          </button>
        </div>

        {error && <div className="error-message">{error}</div>}

        {isPulling && (
          <div className="pull-progress">
            <Loader2 className="spinner" size={20} />
            <span>{pullProgress}</span>
          </div>
        )}

        {mode === "code" && (
          <div className="import-form">
            <div className="form-group">
              <label htmlFor="import-code">Import Code</label>
              <textarea
                id="import-code"
                value={importCode}
                onChange={(e) => setImportCode(e.target.value)}
                placeholder='Paste the import code from Herculean AI platform here...

Example:
{"name":"Email Assistant","description":"Manage your emails","image":"gcr.io/herculean/assistant:latest","port":8081}'
                rows={6}
                disabled={isLoading}
              />
              <p className="form-help">
                Get this code from the "Download Assistant" option on the Herculean AI web platform.
              </p>
            </div>

            <button
              className="btn btn-primary btn-large"
              onClick={handleImportCode}
              disabled={isLoading || !importCode.trim()}
            >
              {isLoading ? (
                <>
                  <Loader2 className="spinner" size={16} />
                  Importing...
                </>
              ) : (
                <>
                  <Download size={16} />
                  Import Assistant
                </>
              )}
            </button>
          </div>
        )}

        {mode === "manual" && (
          <div className="import-form">
            <div className="form-group">
              <label htmlFor="name">Assistant Name *</label>
              <input
                id="name"
                type="text"
                value={manualForm.name}
                onChange={(e) => setManualForm({ ...manualForm, name: e.target.value })}
                placeholder="My Assistant"
                disabled={isLoading}
              />
            </div>

            <div className="form-group">
              <label htmlFor="description">Description</label>
              <input
                id="description"
                type="text"
                value={manualForm.description}
                onChange={(e) => setManualForm({ ...manualForm, description: e.target.value })}
                placeholder="What does this assistant do?"
                disabled={isLoading}
              />
            </div>

            <div className="form-group">
              <label htmlFor="image">Docker Image *</label>
              <input
                id="image"
                type="text"
                value={manualForm.image}
                onChange={(e) => setManualForm({ ...manualForm, image: e.target.value })}
                placeholder="gcr.io/herculean/assistant:latest"
                disabled={isLoading}
              />
              <p className="form-help">
                The Docker image containing your assistant.
              </p>
            </div>

            <div className="form-group">
              <label htmlFor="port">Port (optional)</label>
              <input
                id="port"
                type="number"
                value={manualForm.port || ""}
                onChange={(e) => setManualForm({ ...manualForm, port: parseInt(e.target.value) || 0 })}
                placeholder="Auto-assign"
                min={1024}
                max={65535}
                disabled={isLoading}
              />
              <p className="form-help">
                Leave empty to auto-assign an available port.
              </p>
            </div>

            <button
              className="btn btn-primary btn-large"
              onClick={handleManualAdd}
              disabled={isLoading || !manualForm.name.trim() || !manualForm.image.trim()}
            >
              {isLoading ? (
                <>
                  <Loader2 className="spinner" size={16} />
                  Adding...
                </>
              ) : (
                <>
                  <Plus size={16} />
                  Add Assistant
                </>
              )}
            </button>
          </div>
        )}
      </main>
    </div>
  );
}

export default ImportAssistant;
