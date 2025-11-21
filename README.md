# Tauri Station

A Tauri application for managing printers and handling print jobs via MQTT. This application can run as a service on Windows and macOS, monitoring a folder for ESC/POS command files and sending them via MQTT, as well as receiving print jobs via MQTT.

## Features

- **MQTT Integration**: Connect to an MQTT server to publish and receive print jobs
- **File Watcher**: Monitor a folder for SPL command files (`.spl`) and automatically publish them via MQTT
- **Printer Management**: Configure multiple printers with different types:
  - ESC/POS Receipt printers (with ReceiptLine markdown support)
  - ZPL Sticker printers
  - Office PDF printers
  - PDF generation
- **Station Configuration**: Set MQTT server, station name, and watch folder
- **Printer Testing**: Test each configured printer with appropriate input formats

## Prerequisites

- Node.js (v18 or later)
- Rust (latest stable)
- Tauri CLI: `npm install -g @tauri-apps/cli`

## Installation

1. Install dependencies:
```bash
npm install
```

2. Build the application:
```bash
npm run tauri build
```

3. For development:
```bash
npm run dev
```

## Configuration

### Station Configuration

1. **MQTT Server**: Enter your MQTT broker URL (e.g., `mqtt://localhost:1883`)
2. **Station Name**: Unique name for this station (used in MQTT topics)
3. **Watch Folder**: Folder to monitor for SPL command files (`.spl`)

### Printer Configuration

1. Click "Add Printer"
2. Enter a name for the printer
3. Select printer type:
   - **ESC/POS Receipt**: For thermal receipt printers (supports ReceiptLine markdown)
   - **ZPL Sticker**: For label printers that accept ZPL code
   - **Office Printer**: For printers that print PDF files
   - **PDF**: Generate PDF files (opens in default PDF viewer)
4. Select the system printer from the dropdown

## MQTT Topics

### Publishing (File Watcher)
- **Topic**: `station/{station_name}/escpos`
- **Message**: JSON with `path`, `content` (base64), and `timestamp`

### Subscribing (Print Jobs)
- **Topic**: `station/{station_name}/print`
- **Message**: JSON with:
  - `printer_id`: ID of the printer to use
  - `printer_type`: Type of printer (`escpos`, `zpl`, `office`, `pdf`)
  - `content`: Print content (varies by type)
  - `content_type`: Content encoding (`pdf_base64` for PDFs)

## Testing Printers

### ESC/POS Receipt
Use ReceiptLine markdown format:
```
# Receipt Title
**Bold text**
Normal text
---
Item 1        $10.00
Item 2        $20.00
---
**Total: $30.00**
```

### ZPL Sticker
Use ZPL code:
```
^XA
^FO50,50^A0N,30,30^FDTest Label^FS
^XZ
```

### Office Printer / PDF
Select a PDF file to print or view.

## Running as a Service

### macOS
Create a Launch Agent plist file at `~/Library/LaunchAgents/com.tauri.station.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.tauri.station</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/tauri-station</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load with: `launchctl load ~/Library/LaunchAgents/com.tauri.station.plist`

### Windows
Create a Windows Service using NSSM (Non-Sucking Service Manager) or similar tool.

## License

MIT
