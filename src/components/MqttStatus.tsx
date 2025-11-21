import React from "react";

interface LastFile {
  path: string;
  timestamp: Date;
}

interface MqttStatusProps {
  connected: boolean;
  watching: boolean;
  stationName: string;
  lastFile: LastFile | null;
}

const MqttStatus: React.FC<MqttStatusProps> = ({ connected, watching, stationName, lastFile }) => {
  const formatTime = (date: Date) => {
    return date.toLocaleString();
  };

  const getFileName = (path: string) => {
    const parts = path.split(/[/\\]/);
    return parts[parts.length - 1] || path;
  };

  return (
    <div className="card">
      <h2>Status</h2>
      <div>
        <p>
          <span className={`status-indicator ${connected ? "connected" : "disconnected"}`}></span>
          <strong>WebSocket:</strong> {connected ? "Connected" : "Disconnected"}
        </p>
        <p>
          <span className={`status-indicator ${watching ? "connected" : "disconnected"}`}></span>
          <strong>File Watcher:</strong> {watching ? "Watching" : "Stopped"}
        </p>
        {watching && lastFile && (
          <p>
            <strong>Last File:</strong> {getFileName(lastFile.path)} <br />
            <span style={{ fontSize: "0.9em", color: "#666", marginLeft: "20px" }}>
              {formatTime(lastFile.timestamp)}
            </span>
          </p>
        )}
        {stationName && (
          <p>
            <strong>Station Name:</strong> {stationName}
          </p>
        )}
      </div>
    </div>
  );
};

export default MqttStatus;
