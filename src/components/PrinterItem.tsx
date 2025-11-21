import React from "react";

interface PrinterConfig {
  id: string;
  name: string;
  printer_type: string;
  system_printer: string;
}

interface PrinterItemProps {
  printer: PrinterConfig;
  onEdit: (printer: PrinterConfig) => void;
  onDelete: (printerId: string) => void;
}

const PrinterItem: React.FC<PrinterItemProps> = ({ printer, onEdit, onDelete }) => {
  const printerTypeLabels: Record<string, string> = {
    escpos: "ESC/POS Receipt",
    zpl: "ZPL Sticker",
    office: "Office Printer",
    pdf: "PDF",
  };

  const handleDelete = () => {
    if (confirm(`Are you sure you want to delete "${printer.name}"?`)) {
      onDelete(printer.id);
    }
  };

  return (
    <li className="printer-item">
      <div className="printer-info">
        <div className="printer-name">{printer.name}</div>
        <div className="printer-details">
          Type: {printerTypeLabels[printer.printer_type] || printer.printer_type} | 
          System Printer: {printer.system_printer}
        </div>
      </div>
      <div className="printer-actions">
        <button onClick={() => onEdit(printer)}>Edit</button>
        <button className="danger" onClick={handleDelete}>
          Delete
        </button>
      </div>
    </li>
  );
};

export default PrinterItem;
