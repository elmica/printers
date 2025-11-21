import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import Configuration from "./components/Configuration";
import PrinterManagement from "./components/PrinterManagement";
import MqttStatus from "./components/MqttStatus";

interface StationConfig {
  websocket_url: string;
  station_name: string;
  watch_folder: string;
}

interface PrinterConfig {
  id: string;
  name: string;
  printer_type: string;
  system_printer: string;
}

function App() {
  const [config, setConfig] = useState<StationConfig | null>(null);
  const [printers, setPrinters] = useState<PrinterConfig[]>([]);
  const [_websocket, setWebsocket] = useState<WebSocket | null>(null); // State kept for reference, but we use ref for operations
  const [wsConnected, setWsConnected] = useState(false);
  const [watching, setWatching] = useState(false);
  const [lastFile, setLastFile] = useState<{ path: string; timestamp: Date } | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const websocketRef = useRef<WebSocket | null>(null); // Ref to always get current WebSocket instance

  useEffect(() => {
    // Wait a bit before initializing to ensure Tauri is ready
    const initTimer = setTimeout(() => {
      loadConfig();
      loadPrinters();
      
      // Listen for file detected events
      listen("file-detected", (event) => {
        const { path, content } = event.payload as { path: string; content: string };
        handleFileDetected(path, content);
      }).catch(err => {
        console.error("Failed to listen for file-detected:", err);
      });
    }, 100);

    return () => {
      clearTimeout(initTimer);
    };
  }, []);

  useEffect(() => {
    if (config && config.websocket_url && config.station_name) {
      connectWebSocket();
    } else {
      disconnectWebSocket();
    }

    return () => {
      disconnectWebSocket();
    };
  }, [config?.websocket_url, config?.station_name]);

  useEffect(() => {
    if (config && config.watch_folder && !watching) {
      startWatching();
    }

    return () => {
      if (watching) {
        stopWatching();
      }
    };
  }, [config?.watch_folder]);

  const loadConfig = async () => {
    try {
      const stationConfig = await invoke<StationConfig>("get_station_config");
      setConfig(stationConfig);
    } catch (error) {
      console.error("Failed to load config:", error);
    }
  };

  const loadPrinters = async () => {
    try {
      const printerList = await invoke<PrinterConfig[]>("get_printers");
      setPrinters(printerList);
    } catch (error) {
      console.error("Failed to load printers:", error);
    }
  };

  const connectWebSocket = () => {
    if (!config) return;

    // Clear any existing reconnect timeout
    if (reconnectAttemptsRef.current > 0) {
      console.log(`Reconnect attempt #${reconnectAttemptsRef.current}`);
    }

    // Close existing connection if any
    if (websocketRef.current) {
      websocketRef.current.onerror = null;
      websocketRef.current.onclose = null;
      websocketRef.current.close();
    }
    setWebsocket(null);
    websocketRef.current = null;

    try {
      // Use current config values
      const currentConfig = config;
      let wsUrl = currentConfig.websocket_url.trim();
      
      // Ensure URL starts with ws:// or wss://
      if (!wsUrl.startsWith("ws://") && !wsUrl.startsWith("wss://")) {
        wsUrl = `ws://${wsUrl}`;
      }

      // Remove trailing slashes (can cause connection issues)
      wsUrl = wsUrl.replace(/\/+$/, '');

      try {
        new URL(wsUrl);
      } catch (e) {
        console.error("Invalid WebSocket URL:", wsUrl);
        alert(`Invalid WebSocket URL: ${wsUrl}. Please use ws://host:port or wss://host:port format.`);
        return;
      }

      console.log(`🔗 Attempting to connect to WebSocket: ${wsUrl}`);
      
      const ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        console.log("✅ WebSocket connected successfully to", wsUrl);
        console.log("WebSocket readyState:", ws.readyState, "(should be 2=OPEN)");
        setWsConnected(true);
        reconnectAttemptsRef.current = 0; // Reset reconnect attempts on successful connection
        
        // Send registration message with station name
        if (currentConfig && currentConfig.station_name) {
          try {
            const registerMessage = {
              type: "register",
              station_name: currentConfig.station_name,
            };
            ws.send(JSON.stringify(registerMessage));
            console.log(`📤 Registered station: ${currentConfig.station_name}`);
            
            // Verify connection is still open after sending
            setTimeout(() => {
              if (ws.readyState === WebSocket.OPEN) {
                console.log("✅ Connection still open after registration");
              } else {
                console.warn("⚠️ Connection state changed after registration:", ws.readyState);
              }
            }, 100);
          } catch (error) {
            console.error("Failed to send registration message:", error);
          }
        }
      };

      ws.onmessage = async (event) => {
        try {
          const message = JSON.parse(event.data);
          console.log("📨 Received WebSocket message:", message);
          console.log("WebSocket readyState:", ws.readyState, "(1=CONNECTING, 2=OPEN, 3=CLOSING, 4=CLOSED)");
          
          // Reload printers to ensure we have the latest list
          const currentPrinters = await invoke<PrinterConfig[]>("get_printers");
          console.log("Current printers available:", currentPrinters.map(p => ({ id: p.id, name: p.name })));
          
          // Use the current WebSocket instance and latest printers
          handleWebSocketMessage(message, ws, currentPrinters);
        } catch (error) {
          console.error("Failed to parse WebSocket message:", error);
          console.log("Raw message data:", event.data);
        }
      };

      ws.onerror = (error) => {
        console.error("❌ WebSocket error event:", error);
        console.error("WebSocket readyState:", ws.readyState);
        setWsConnected(false);
      };

      ws.onclose = (event) => {
        const closeCodes: { [key: number]: string } = {
          1000: "Normal Closure",
          1001: "Going Away",
          1002: "Protocol Error",
          1003: "Unsupported Data",
          1006: "Abnormal Closure (no close frame)",
          1007: "Invalid Data",
          1008: "Policy Violation",
          1009: "Message Too Big",
          1010: "Missing Extension",
          1011: "Internal Error",
        };
        
        const closeReason = closeCodes[event.code] || `Unknown (${event.code})`;
        console.log(`🔌 WebSocket disconnected:`);
        console.log(`   Code: ${event.code} (${closeReason})`);
        console.log(`   Reason: ${event.reason || "No reason provided"}`);
        console.log(`   Was Clean: ${event.wasClean}`);
        console.log(`   Current readyState: ${ws.readyState}`);
        
        setWsConnected(false);
        
        // Only attempt to reconnect if:
        // 1. Config is still valid
        // 2. It wasn't a manual close (code 1000)
        // 3. We haven't exceeded max reconnect attempts
        if (event.code !== 1000 && currentConfig && currentConfig.websocket_url && currentConfig.station_name) {
          const maxAttempts = 10;
          reconnectAttemptsRef.current += 1;
          
          if (reconnectAttemptsRef.current <= maxAttempts) {
            const delay = Math.min(5000 * reconnectAttemptsRef.current, 30000); // Exponential backoff, max 30s
            console.log(`⏳ Will attempt to reconnect in ${delay/1000} seconds (attempt ${reconnectAttemptsRef.current}/${maxAttempts})...`);
            
            setTimeout(() => {
              console.log("🔄 Attempting to reconnect WebSocket...");
              connectWebSocket();
            }, delay);
          } else {
            console.error(`❌ Max reconnect attempts (${maxAttempts}) reached. Please check:`);
            console.error(`   1. Is Node-RED running?`);
            console.error(`   2. Is the WebSocket server listening on ${wsUrl}?`);
            console.error(`   3. Check Node-RED WebSocket node configuration`);
            reconnectAttemptsRef.current = 0; // Reset for next manual connection attempt
          }
        } else if (event.code === 1000) {
          console.log("Connection closed normally (manual close)");
        } else {
          console.log("Not attempting to reconnect - invalid config or manual close");
        }
      };
      
      setWebsocket(ws);
      websocketRef.current = ws; // Keep ref in sync
    } catch (error: any) {
      console.error("Failed to create WebSocket connection:", error);
      console.error("Error stack:", error.stack);
      setWsConnected(false);
      reconnectAttemptsRef.current = 0;
      alert(`Failed to connect to WebSocket: ${error.message || error}\n\nPlease check:\n1. Is Node-RED running?\n2. Is the WebSocket server configured correctly?\n3. Check the URL: ${config.websocket_url}`);
    }
  };

  const disconnectWebSocket = () => {
    reconnectAttemptsRef.current = 0; // Reset reconnect attempts
    if (websocketRef.current) {
      websocketRef.current.onerror = null;
      websocketRef.current.onclose = null;
      websocketRef.current.close(1000, "Manual disconnect"); // Normal closure code
    }
    setWebsocket(null);
    websocketRef.current = null;
    setWsConnected(false);
  };

  const handleWebSocketMessage = async (message: any, wsInstance: WebSocket, currentPrinters: PrinterConfig[]) => {
    try {
      console.log("Processing message type:", message.type);
      
      // Handle different message types
      if (message.type === "print_job") {
        const { printer_id, printer_type, content, content_type } = message;
        console.log("Print job received:", { printer_id, printer_type, content_type });

        const printer = currentPrinters.find((p) => p.id === printer_id);
        if (!printer) {
          console.error(`Printer ${printer_id} not found. Available printers:`, currentPrinters.map(p => p.id));
          return;
        }

        console.log(`Printing to printer: ${printer.name} (${printer.system_printer})`);

        switch (printer_type) {
          case "escpos":
            await invoke("test_print_escpos", {
              printerName: printer.system_printer,
              receiptlineMarkdown: content,
            });
            console.log("✅ ESC/POS print job completed");
            break;
          case "zpl":
            await invoke("test_print_zpl", {
              printerName: printer.system_printer,
              zplCode: content,
            });
            console.log("✅ ZPL print job completed");
            break;
          case "office":
            if (content_type === "pdf_base64") {
              await invoke("test_print_office", {
                printerName: printer.system_printer,
                pdfBase64: content,
              });
              console.log("✅ Office print job completed");
            } else {
              console.warn("Office printer requires pdf_base64 content_type");
            }
            break;
          case "pdf":
            if (content_type === "pdf_base64") {
              await invoke("test_print_pdf", {
                pdfBase64: content,
              });
              console.log("✅ PDF print job completed");
            } else {
              console.warn("PDF printer requires pdf_base64 content_type");
            }
            break;
          default:
            console.warn("Unknown printer type:", printer_type);
        }
      } else if (message.type === "ping") {
        // Respond to ping with pong
        console.log("Ping received, sending pong");
        if (wsInstance && wsInstance.readyState === WebSocket.OPEN) {
          wsInstance.send(JSON.stringify({ type: "pong" }));
          console.log("✅ Pong sent");
        } else {
          console.warn("Cannot send pong - WebSocket not open (readyState:", wsInstance?.readyState, ")");
        }
      } else {
        console.log("Unhandled message type:", message.type, "- Full message:", message);
      }
    } catch (error) {
      console.error("Failed to handle WebSocket message:", error);
      console.error("Message that caused error:", message);
    }
  };

  const handleFileDetected = async (path: string, content: string) => {
    // Update last file info
    const now = new Date();
    setLastFile({ path, timestamp: now });

    // Use ref to get current WebSocket instance (always up-to-date)
    const currentWs = websocketRef.current;
    const currentConfig = config;

    if (!currentWs || !wsConnected || !currentConfig) {
      console.log("File detected but WebSocket not connected:", path);
      console.log("  - WebSocket exists:", !!currentWs);
      console.log("  - wsConnected:", wsConnected);
      console.log("  - Config exists:", !!currentConfig);
      return;
    }

    try {
      const message = {
        type: "file_detected",
        station_name: currentConfig.station_name,
        path,
        content,
        timestamp: now.toISOString(),
      };

      const readyState = currentWs.readyState;
      console.log(`Attempting to send file ${path}, WebSocket readyState: ${readyState} (1=CONNECTING, 2=OPEN, 3=CLOSING, 4=CLOSED)`);

      if (readyState === WebSocket.OPEN) {
        currentWs.send(JSON.stringify(message));
        console.log(`✅ File data sent via WebSocket: ${path}`);
      } else {
        console.error(`❌ WebSocket not open (readyState: ${readyState}), cannot send file data for: ${path}`);
        if (readyState === WebSocket.CLOSING) {
          console.error("  WebSocket is closing - connection may be dropping");
        } else if (readyState === WebSocket.CLOSED) {
          console.error("  WebSocket is closed - connection lost");
        } else if (readyState === WebSocket.CONNECTING) {
          console.error("  WebSocket is still connecting - wait and retry");
        }
      }
    } catch (error) {
      console.error("❌ Failed to handle file:", error);
      console.error("  File path:", path);
      console.error("  Error details:", error);
    }
  };

  const startWatching = async () => {
    if (!config || !config.watch_folder) return;

    try {
      await invoke("start_file_watcher", { folderPath: config.watch_folder });
      setWatching(true);
    } catch (error) {
      console.error("Failed to start watcher:", error);
    }
  };

  const stopWatching = async () => {
    try {
      await invoke("stop_file_watcher");
      setWatching(false);
    } catch (error) {
      console.error("Failed to stop watcher:", error);
    }
  };

  const handleConfigSave = async (newConfig: StationConfig) => {
    try {
      await invoke("save_station_config", { config: newConfig });
      setConfig(newConfig);
    } catch (error) {
      console.error("Failed to save config:", error);
      alert("Failed to save configuration");
    }
  };

  const handlePrinterSave = async (printer: PrinterConfig) => {
    try {
      await invoke("save_printer", { printer });
      await loadPrinters();
    } catch (error) {
      console.error("Failed to save printer:", error);
      alert("Failed to save printer");
    }
  };

  const handlePrinterDelete = async (printerId: string) => {
    try {
      await invoke("delete_printer", { printerId });
      await loadPrinters();
    } catch (error) {
      console.error("Failed to delete printer:", error);
      alert("Failed to delete printer");
    }
  };

  return (
    <div className="container">
      <h1>Tauri Station</h1>
      
      <MqttStatus 
        connected={wsConnected}
        watching={watching}
        stationName={config?.station_name || ""}
        lastFile={lastFile}
      />

      <Configuration
        config={config}
        onSave={handleConfigSave}
      />

      <PrinterManagement
        printers={printers}
        onSave={handlePrinterSave}
        onDelete={handlePrinterDelete}
      />
    </div>
  );
}

export default App;
