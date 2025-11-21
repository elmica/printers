import React, { useState, useEffect } from "react";
import PrinterTest from "./PrinterTest";

interface PrinterConfig {
  id: string;
  name: string;
  printer_type: string;
  system_printer: string;
}

interface PrinterFormProps {
  printer: PrinterConfig | null;
  systemPrinters: string[];
  onSave: (printer: PrinterConfig) => void;
  onCancel: () => void;
}

const PrinterForm: React.FC<PrinterFormProps> = ({
  printer,
  systemPrinters,
  onSave,
  onCancel,
}) => {
  const [formData, setFormData] = useState<PrinterConfig>({
    id: "",
    name: "",
    printer_type: "escpos",
    system_printer: "",
  });

  useEffect(() => {
    if (printer) {
      setFormData(printer);
    }
  }, [printer]);

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value,
    });
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.name || !formData.system_printer) {
      alert("Please fill in all required fields");
      return;
    }
    onSave(formData);
  };

  const printerTypes = [
    { value: "escpos", label: "ESC/POS Receipt" },
    { value: "zpl", label: "ZPL Sticker" },
    { value: "office", label: "Office Printer" },
    { value: "pdf", label: "PDF" },
  ];

  return (
    <div style={{ marginBottom: "20px", padding: "20px", border: "1px solid #ddd", borderRadius: "4px" }}>
      <h3>{printer ? "Edit Printer" : "Add Printer"}</h3>
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="name">Printer Name *</label>
          <input
            type="text"
            id="name"
            name="name"
            value={formData.name}
            onChange={handleChange}
            placeholder="e.g., Receipt Printer 1"
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="printer_type">Printer Type *</label>
          <select
            id="printer_type"
            name="printer_type"
            value={formData.printer_type}
            onChange={handleChange}
            required
          >
            {printerTypes.map((type) => (
              <option key={type.value} value={type.value}>
                {type.label}
              </option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="system_printer">System Printer *</label>
          <select
            id="system_printer"
            name="system_printer"
            value={formData.system_printer}
            onChange={handleChange}
            required
          >
            <option value="">Select a printer</option>
            {systemPrinters.map((printer) => (
              <option key={printer} value={printer}>
                {printer}
              </option>
            ))}
          </select>
        </div>

        <div style={{ display: "flex", gap: "10px" }}>
          <button type="submit">Save</button>
          <button type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>

      {formData.name && formData.system_printer && (
        <PrinterTest
          printer={formData}
        />
      )}
    </div>
  );
};

export default PrinterForm;
