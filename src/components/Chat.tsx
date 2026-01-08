import { useState, useEffect, useRef } from "react";
import { ArrowLeft, Send, Monitor, Loader2 } from "lucide-react";
import type { Assistant } from "../App";
import "./Chat.css";

interface ChatProps {
  assistant: Assistant;
  onBack: () => void;
}

interface Message {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: Date;
}

function Chat({ assistant, onBack }: ChatProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isConnected, setIsConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [showVnc, setShowVnc] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Connect to WebSocket
  useEffect(() => {
    connectWebSocket();
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [assistant]);

  const connectWebSocket = () => {
    const wsUrl = `ws://localhost:${assistant.port}/ws`;
    console.log("Connecting to WebSocket:", wsUrl);

    try {
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log("WebSocket connected");
        setIsConnected(true);
        addMessage({
          role: "system",
          content: `Connected to ${assistant.name}. You can start chatting now.`,
        });
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.type === "response" || data.type === "message") {
            addMessage({
              role: "assistant",
              content: data.content || data.message || JSON.stringify(data),
            });
            setIsLoading(false);
          } else if (data.type === "thinking" || data.type === "status") {
            // Handle thinking/status updates
            console.log("Status:", data);
          } else if (data.type === "error") {
            addMessage({
              role: "system",
              content: `Error: ${data.message || data.error}`,
            });
            setIsLoading(false);
          }
        } catch {
          // Plain text message
          addMessage({
            role: "assistant",
            content: event.data,
          });
          setIsLoading(false);
        }
      };

      ws.onerror = (error) => {
        console.error("WebSocket error:", error);
        setIsConnected(false);
        addMessage({
          role: "system",
          content: "Connection error. Please check if the assistant is running.",
        });
      };

      ws.onclose = () => {
        console.log("WebSocket disconnected");
        setIsConnected(false);
      };
    } catch (error) {
      console.error("Failed to connect WebSocket:", error);
      setIsConnected(false);
    }
  };

  const addMessage = (msg: Omit<Message, "id" | "timestamp">) => {
    setMessages((prev) => [
      ...prev,
      {
        ...msg,
        id: crypto.randomUUID(),
        timestamp: new Date(),
      },
    ]);
  };

  const handleSend = () => {
    if (!input.trim() || !wsRef.current || !isConnected) return;

    const userMessage = input.trim();
    addMessage({ role: "user", content: userMessage });
    setInput("");
    setIsLoading(true);

    // Send message through WebSocket
    wsRef.current.send(JSON.stringify({
      type: "message",
      content: userMessage,
    }));
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="chat-container">
      <header className="chat-header">
        <button className="btn btn-icon" onClick={onBack}>
          <ArrowLeft size={20} />
        </button>
        <div className="chat-title">
          <span className="assistant-icon">{assistant.icon || "🤖"}</span>
          <div>
            <h2>{assistant.name}</h2>
            <span className={`connection-status ${isConnected ? "connected" : "disconnected"}`}>
              {isConnected ? "Connected" : "Disconnected"}
            </span>
          </div>
        </div>
        {assistant.vnc_enabled && (
          <button
            className={`btn btn-secondary ${showVnc ? "active" : ""}`}
            onClick={() => setShowVnc(!showVnc)}
          >
            <Monitor size={16} />
            {showVnc ? "Hide Screen" : "Show Screen"}
          </button>
        )}
      </header>

      <div className="chat-main">
        <div className={`messages-container ${showVnc ? "with-vnc" : ""}`}>
          <div className="messages">
            {messages.length === 0 && (
              <div className="welcome-message">
                <span className="welcome-icon">{assistant.icon || "🤖"}</span>
                <h3>Welcome to {assistant.name}</h3>
                <p>{assistant.description}</p>
                <p className="hint">Type a message below to get started.</p>
              </div>
            )}

            {messages.map((msg) => (
              <div key={msg.id} className={`message ${msg.role}`}>
                <div className="message-content">
                  {msg.content}
                </div>
                <div className="message-time">
                  {msg.timestamp.toLocaleTimeString()}
                </div>
              </div>
            ))}

            {isLoading && (
              <div className="message assistant loading">
                <Loader2 className="spinner" size={16} />
                <span>Thinking...</span>
              </div>
            )}

            <div ref={messagesEndRef} />
          </div>
        </div>

        {showVnc && assistant.vnc_enabled && (
          <div className="vnc-container">
            <iframe
              src={`http://localhost:${assistant.port}/novnc/vnc.html?autoconnect=true&resize=scale`}
              title="VNC Viewer"
              className="vnc-iframe"
            />
          </div>
        )}
      </div>

      <div className="chat-input-container">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder={isConnected ? "Type a message..." : "Connecting..."}
          disabled={!isConnected}
          rows={1}
        />
        <button
          className="btn btn-primary send-btn"
          onClick={handleSend}
          disabled={!isConnected || !input.trim()}
        >
          <Send size={20} />
        </button>
      </div>
    </div>
  );
}

export default Chat;
