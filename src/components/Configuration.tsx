import React, { useState, useEffect } from "react";
import { open } from "@tauri-apps/api/dialog";

interface StationConfig {
  websocket_url: string;
  station_name: string;
  watch_folder: string;
}

interface ConfigurationProps {
  config: StationConfig | null;
  onSave: (config: StationConfig) => void;
}

const Configuration: React.FC<ConfigurationProps> = ({ config, onSave }) => {
  const [formData, setFormData] = useState<StationConfig>({
    websocket_url: "",
    station_name: "",
    watch_folder: "",
  });

  useEffect(() => {
    if (config) {
      setFormData(config);
    }
  }, [config]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value,
    });
  };

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      
      if (selected && typeof selected === "string") {
        setFormData({
          ...formData,
          watch_folder: selected,
        });
      }
    } catch (error) {
      console.error("Failed to select folder:", error);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(formData);
  };

  return (
    <div className="card">
      <h2>Configuration</h2>
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="websocket_url">WebSocket Server</label>
          <input
            type="text"
            id="websocket_url"
            name="websocket_url"
            value={formData.websocket_url}
            onChange={handleChange}
            placeholder="ws://localhost:1880 (Node-RED default) or ws://localhost:8080"
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="station_name">Station Name</label>
          <input
            type="text"
            id="station_name"
            name="station_name"
            value={formData.station_name}
            onChange={handleChange}
            placeholder="Station-1"
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="watch_folder">Watch Folder</label>
          <div style={{ display: "flex", gap: "10px" }}>
            <input
              type="text"
              id="watch_folder"
              name="watch_folder"
              value={formData.watch_folder}
              onChange={handleChange}
              placeholder="Select folder to watch for .spl files"
              readOnly
            />
            <button type="button" onClick={handleSelectFolder}>
              Browse
            </button>
          </div>
        </div>

        <button type="submit">Save Configuration</button>
      </form>
    </div>
  );
};

export default Configuration;
