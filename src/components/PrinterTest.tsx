import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface PrinterConfig {
  id: string;
  name: string;
  printer_type: string;
  system_printer: string;
}

interface PrinterTestProps {
  printer: PrinterConfig;
}

const PrinterTest: React.FC<PrinterTestProps> = ({ printer }) => {
  const [testInput, setTestInput] = useState("");
  const [testing, setTesting] = useState(false);
  const [testFile, setTestFile] = useState<File | null>(null);

  const handleTest = async () => {
    if (!testInput.trim() && !testFile) {
      alert("Please provide test input");
      return;
    }

    setTesting(true);
    try {
      switch (printer.printer_type) {
        case "escpos":
          await invoke("test_print_escpos", {
            printerName: printer.system_printer,
            receiptlineMarkdown: testInput,
          });
          alert("Print job sent!");
          break;
        case "zpl":
          await invoke("test_print_zpl", {
            printerName: printer.system_printer,
            zplCode: testInput,
          });
          alert("Print job sent!");
          break;
        case "office":
          if (testFile) {
            const reader = new FileReader();
            reader.onload = async (e) => {
              const base64 = (e.target?.result as string).split(",")[1];
              try {
                await invoke("test_print_office", {
                  printerName: printer.system_printer,
                  pdfBase64: base64,
                });
                alert("Print job sent!");
              } catch (error) {
                console.error("Print error:", error);
                alert("Failed to print: " + error);
              }
              setTesting(false);
            };
            reader.readAsDataURL(testFile);
            return;
          } else {
            alert("Please select a PDF file");
          }
          break;
        case "pdf":
          if (testFile) {
            const reader = new FileReader();
            reader.onload = async (e) => {
              const base64 = (e.target?.result as string).split(",")[1];
              try {
                await invoke("test_print_pdf", {
                  pdfBase64: base64,
                });
                alert("PDF opened!");
              } catch (error) {
                console.error("PDF error:", error);
                alert("Failed to open PDF: " + error);
              }
              setTesting(false);
            };
            reader.readAsDataURL(testFile);
            return;
          } else {
            alert("Please select a PDF file");
          }
          break;
      }
    } catch (error) {
      console.error("Test error:", error);
      alert("Failed to test printer: " + error);
    }
    setTesting(false);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setTestFile(file);
    }
  };

  const getPlaceholder = () => {
    switch (printer.printer_type) {
      case "escpos":
        return `# Receipt Title
**Bold text**
Normal text
---
Item 1        $10.00
Item 2        $20.00
---
**Total: $30.00**`;
      case "zpl":
        return `^XA
^FO50,50^A0N,30,30^FDTest Label^FS
^XZ`;
      default:
        return "";
    }
  };

  return (
    <div className="test-section">
      <h3>Test Printer</h3>
      {(printer.printer_type === "escpos" || printer.printer_type === "zpl") && (
        <div className="form-group">
          <label>
            {printer.printer_type === "escpos" ? "ReceiptLine Markdown" : "ZPL Code"}
          </label>
          <textarea
            value={testInput}
            onChange={(e) => setTestInput(e.target.value)}
            placeholder={getPlaceholder()}
            rows={10}
          />
        </div>
      )}
      {(printer.printer_type === "office" || printer.printer_type === "pdf") && (
        <div className="form-group">
          <label>PDF File</label>
          <input
            type="file"
            accept=".pdf"
            onChange={handleFileChange}
          />
          {testFile && <p>Selected: {testFile.name}</p>}
        </div>
      )}
      <button onClick={handleTest} disabled={testing}>
        {testing ? "Testing..." : "Test Print"}
      </button>
    </div>
  );
};

export default PrinterTest;
