import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { Download, ExternalLink, RefreshCw, Check, AlertCircle } from "lucide-react";
import type { DockerStatus } from "../App";
import "./DockerSetup.css";

interface DockerSetupProps {
  dockerStatus: DockerStatus | null;
  onReady: () => void;
}

type SetupStep = "check" | "download" | "install" | "start" | "ready";

function DockerSetup({ dockerStatus, onReady }: DockerSetupProps) {
  const [step, setStep] = useState<SetupStep>(getInitialStep());
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  function getInitialStep(): SetupStep {
    if (!dockerStatus) return "check";
    if (!dockerStatus.installed) return "download";
    if (!dockerStatus.running) return "start";
    return "ready";
  }

  const handleDownloadDocker = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const url = await invoke<string>("get_docker_download_url");
      await open(url);
      setStep("install");
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleStartDocker = async () => {
    setIsLoading(true);
    setError(null);
    try {
      await invoke("start_docker");
      setStep("start");
      // Wait for Docker to be ready
      await invoke("wait_for_docker");
      setStep("ready");
      setTimeout(onReady, 1000);
    } catch (err) {
      setError(String(err));
      setIsLoading(false);
    }
  };

  const handleCheckAgain = async () => {
    setIsLoading(true);
    setError(null);
    try {
      onReady(); // This will trigger re-check in parent
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="docker-setup">
      <div className="setup-container">
        <div className="setup-header">
          <h1>Welcome to Herculean Desktop</h1>
          <p>Let's get you set up to run AI assistants locally.</p>
        </div>

        <div className="setup-steps">
          {/* Step 1: Docker Not Installed */}
          {step === "download" && (
            <div className="setup-step">
              <div className="step-icon warning">
                <AlertCircle size={32} />
              </div>
              <h2>Docker Desktop Required</h2>
              <p>
                Docker Desktop is needed to run AI assistants locally. It's free and
                takes just a few minutes to install.
              </p>

              {error && <div className="error-message">{error}</div>}

              <div className="step-actions">
                <button
                  className="btn btn-primary btn-large"
                  onClick={handleDownloadDocker}
                  disabled={isLoading}
                >
                  {isLoading ? (
                    <>
                      <div className="spinner-small" />
                      Opening download...
                    </>
                  ) : (
                    <>
                      <Download size={20} />
                      Download Docker Desktop
                    </>
                  )}
                </button>
              </div>

              <p className="step-note">
                After installing, click "I've Installed Docker" below.
              </p>

              <button
                className="btn btn-secondary"
                onClick={handleCheckAgain}
                disabled={isLoading}
              >
                <RefreshCw size={16} />
                I've Installed Docker
              </button>
            </div>
          )}

          {/* Step 2: Installing Docker */}
          {step === "install" && (
            <div className="setup-step">
              <div className="step-icon">
                <Download size={32} />
              </div>
              <h2>Installing Docker Desktop</h2>
              <p>
                Follow the Docker Desktop installer to complete the installation.
                Once finished, click the button below.
              </p>

              <div className="install-instructions">
                <h3>Installation Steps:</h3>
                <ol>
                  <li>Open the downloaded Docker installer</li>
                  <li>Follow the installation wizard</li>
                  <li>Grant any permissions requested</li>
                  <li>Wait for Docker to finish setting up</li>
                </ol>
              </div>

              {error && <div className="error-message">{error}</div>}

              <div className="step-actions">
                <button
                  className="btn btn-primary"
                  onClick={handleCheckAgain}
                  disabled={isLoading}
                >
                  {isLoading ? (
                    <>
                      <div className="spinner-small" />
                      Checking...
                    </>
                  ) : (
                    <>
                      <Check size={16} />
                      I've Finished Installing
                    </>
                  )}
                </button>

                <button
                  className="btn btn-secondary"
                  onClick={handleDownloadDocker}
                  disabled={isLoading}
                >
                  <ExternalLink size={16} />
                  Download Again
                </button>
              </div>
            </div>
          )}

          {/* Step 3: Docker Installed but Not Running */}
          {step === "start" && (
            <div className="setup-step">
              <div className="step-icon success">
                <Check size={32} />
              </div>
              <h2>Docker Desktop Installed!</h2>
              <p>
                Docker is installed but not running. Let's start it up.
              </p>

              {error && <div className="error-message">{error}</div>}

              <div className="step-actions">
                <button
                  className="btn btn-primary btn-large"
                  onClick={handleStartDocker}
                  disabled={isLoading}
                >
                  {isLoading ? (
                    <>
                      <div className="spinner-small" />
                      Starting Docker...
                    </>
                  ) : (
                    <>
                      <RefreshCw size={20} />
                      Start Docker Desktop
                    </>
                  )}
                </button>
              </div>

              <p className="step-note">
                Docker may take 30-60 seconds to fully start.
              </p>
            </div>
          )}

          {/* Step 4: Ready */}
          {step === "ready" && (
            <div className="setup-step">
              <div className="step-icon success">
                <Check size={32} />
              </div>
              <h2>All Set!</h2>
              <p>Docker is running and ready. Launching the dashboard...</p>
              <div className="spinner"></div>
            </div>
          )}

          {/* Check Step */}
          {step === "check" && (
            <div className="setup-step">
              <div className="spinner"></div>
              <h2>Checking System Requirements</h2>
              <p>Please wait while we check your system...</p>
            </div>
          )}
        </div>

        <footer className="setup-footer">
          <p>
            Need help?{" "}
            <a
              href="https://docs.docker.com/desktop/"
              target="_blank"
              rel="noopener noreferrer"
            >
              Docker Documentation
            </a>
          </p>
        </footer>
      </div>
    </div>
  );
}

export default DockerSetup;
