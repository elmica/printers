import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { v4 as uuidv4 } from "uuid";
import PrinterForm from "./PrinterForm";
import PrinterItem from "./PrinterItem";

interface PrinterConfig {
  id: string;
  name: string;
  printer_type: string;
  system_printer: string;
}

interface PrinterManagementProps {
  printers: PrinterConfig[];
  onSave: (printer: PrinterConfig) => void;
  onDelete: (printerId: string) => void;
}

const PrinterManagement: React.FC<PrinterManagementProps> = ({
  printers,
  onSave,
  onDelete,
}) => {
  const [showForm, setShowForm] = useState(false);
  const [editingPrinter, setEditingPrinter] = useState<PrinterConfig | null>(null);
  const [systemPrinters, setSystemPrinters] = useState<string[]>([]);
  const [loadingPrinters, setLoadingPrinters] = useState(true);
  const [printerError, setPrinterError] = useState<string | null>(null);

  useEffect(() => {
    loadSystemPrinters();
  }, []);

  const loadSystemPrinters = async () => {
    setLoadingPrinters(true);
    setPrinterError(null);
    try {
      console.log("Loading system printers...");
      const printers = await invoke<string[]>("get_system_printers");
      console.log("Loaded system printers:", printers);
      setSystemPrinters(printers);
      if (printers.length === 0) {
        setPrinterError("No printers found on this system. Make sure you have printers installed in Windows.");
      }
    } catch (error: any) {
      const errorMessage = error?.message || error?.toString() || "Unknown error";
      console.error("Failed to load system printers:", error);
      setPrinterError(`Failed to load printers: ${errorMessage}`);
      // Set empty array so the form can still be shown
      setSystemPrinters([]);
    } finally {
      setLoadingPrinters(false);
    }
  };

  const handleAdd = () => {
    setEditingPrinter(null);
    setShowForm(true);
  };

  const handleEdit = (printer: PrinterConfig) => {
    setEditingPrinter(printer);
    setShowForm(true);
  };

  const handleSave = (printer: PrinterConfig) => {
    const printerToSave = {
      ...printer,
      id: printer.id || uuidv4(),
    };
    onSave(printerToSave);
    setShowForm(false);
    setEditingPrinter(null);
  };

  const handleCancel = () => {
    setShowForm(false);
    setEditingPrinter(null);
  };

  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "20px" }}>
        <h2>Printers</h2>
        <button onClick={handleAdd}>Add Printer</button>
      </div>

      {printerError && (
        <div style={{ 
          padding: "10px", 
          marginBottom: "10px", 
          backgroundColor: "#fee", 
          border: "1px solid #fcc",
          borderRadius: "4px",
          color: "#c33"
        }}>
          <strong>Warning:</strong> {printerError}
          <button 
            onClick={loadSystemPrinters} 
            style={{ 
              marginLeft: "10px", 
              padding: "4px 8px",
              cursor: "pointer"
            }}
          >
            Retry
          </button>
        </div>
      )}

      {loadingPrinters && (
        <div style={{ padding: "10px", marginBottom: "10px", color: "#666" }}>
          Loading system printers...
        </div>
      )}

      {showForm && (
        <PrinterForm
          printer={editingPrinter}
          systemPrinters={systemPrinters}
          onSave={handleSave}
          onCancel={handleCancel}
        />
      )}

      <ul className="printer-list">
        {printers.map((printer) => (
          <PrinterItem
            key={printer.id}
            printer={printer}
            onEdit={handleEdit}
            onDelete={onDelete}
          />
        ))}
        {printers.length === 0 && (
          <li style={{ padding: "20px", textAlign: "center", color: "#666" }}>
            No printers configured. Click "Add Printer" to get started.
          </li>
        )}
      </ul>
    </div>
  );
};

export default PrinterManagement;
