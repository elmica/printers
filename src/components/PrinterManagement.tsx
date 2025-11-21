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

  useEffect(() => {
    loadSystemPrinters();
  }, []);

  const loadSystemPrinters = async () => {
    try {
      const printers = await invoke<string[]>("get_system_printers");
      setSystemPrinters(printers);
    } catch (error) {
      console.error("Failed to load system printers:", error);
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
