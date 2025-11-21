use std::process::Command;
use printers::{get_printer_by_name};
use printers::common::base::job::PrinterJobOptions;

pub fn print_escpos_receiptline(printer_name: &str, receiptline_markdown: &str) -> Result<(), String> {
    // Convert ReceiptLine markdown to ESC/POS
    let escpos_data = convert_receiptline_to_escpos(receiptline_markdown)?;
    send_to_printer(printer_name, &escpos_data)
}

pub fn print_zpl(printer_name: &str, zpl_code: &str) -> Result<(), String> {
    send_to_printer(printer_name, zpl_code.as_bytes())
}

pub fn print_office_pdf(printer_name: &str, pdf_base64: &str) -> Result<(), String> {
    use base64::{Engine as _, engine::general_purpose};
    let pdf_data = general_purpose::STANDARD.decode(pdf_base64)
        .map_err(|e| format!("Failed to decode PDF: {}", e))?;
    
    #[cfg(target_os = "windows")]
    {
        // Windows: Use rundll32 to print PDF silently
        use std::fs;
        use std::os::windows::process::CommandExt;
        
        let temp_file = std::env::temp_dir().join(format!("print_{}.pdf", uuid::Uuid::new_v4().as_simple()));
        fs::write(&temp_file, &pdf_data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        // Use rundll32 to print PDF to specific printer
        // Alternative: Use PrintUI.dll or ShellExecute with print verb
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let output = Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-Command",
                &format!("$printer = Get-Printer -Name '{}'; if ($printer) {{ Start-Process -FilePath '{}' -Verb PrintTo -ArgumentList $printer.Name -WindowStyle Hidden -PassThru | Out-Null }} else {{ Write-Error 'Printer not found' }}", 
                    printer_name, temp_file.to_str().unwrap())
            ])
            .output()
            .map_err(|e| format!("Failed to print: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Print failed: {}", stderr));
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::fs;
        let temp_file = std::env::temp_dir().join(format!("print_{}.pdf", uuid::Uuid::new_v4().as_simple()));
        fs::write(&temp_file, &pdf_data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        let output = Command::new("lp")
            .args(&["-d", printer_name, temp_file.to_str().unwrap()])
            .output()
            .map_err(|e| format!("Failed to print: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Print failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err("Printing not supported on this platform".to_string())
    }
}

pub fn print_pdf(pdf_base64: &str) -> Result<(), String> {
    use base64::{Engine as _, engine::general_purpose};
    let pdf_data = general_purpose::STANDARD.decode(pdf_base64)
        .map_err(|e| format!("Failed to decode PDF: {}", e))?;
    
    use std::fs;
    let temp_file = std::env::temp_dir().join(format!("print_{}.pdf", uuid::Uuid::new_v4()));
    fs::write(&temp_file, &pdf_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-Command",
                &format!("Start-Process -FilePath '{}' -WindowStyle Hidden", 
                    temp_file.to_str().unwrap())
            ])
            .output()
            .map_err(|e| format!("Failed to open PDF: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(temp_file.to_str().unwrap())
            .output()
            .map_err(|e| format!("Failed to open PDF: {}", e))?;
    }
    
    Ok(())
}

fn send_to_printer(printer_name: &str, data: &[u8]) -> Result<(), String> {
    // Get the printer by name
    let printer = get_printer_by_name(printer_name)
        .ok_or_else(|| format!("Printer '{}' not found", printer_name))?;
    
    // Print raw data to the printer with raw format
    let job_name = "Tauri Station Print Job";
    let raw_props = [
        ("document-format", "application/vnd.cups-raw"),
    ];
    let job_options = PrinterJobOptions {
        name: Some(job_name),
        raw_properties: &raw_props,
    };
    
    printer
        .print(data, job_options)
        .map_err(|e| format!("Failed to print: {}", e))?;
    
    Ok(())
}

fn convert_receiptline_to_escpos(receiptline_markdown: &str) -> Result<Vec<u8>, String> {
    // Basic ReceiptLine markdown to ESC/POS conversion
    // This is a simplified version - you may want to use a proper library
    let mut escpos = Vec::new();
    
    // ESC/POS initialization
    escpos.extend_from_slice(&[0x1B, 0x40]); // Initialize printer
    
    for line in receiptline_markdown.lines() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            escpos.push(0x0A); // Line feed
            continue;
        }
        
        // Check for formatting
        if trimmed.starts_with("# ") {
            // Large text
            escpos.extend_from_slice(&[0x1D, 0x21, 0x10]); // Double width and height
            escpos.extend_from_slice(trimmed[2..].trim().as_bytes());
            escpos.extend_from_slice(&[0x1D, 0x21, 0x00]); // Normal text
        } else if trimmed.starts_with("**") && trimmed.ends_with("**") {
            // Bold
            escpos.extend_from_slice(&[0x1B, 0x45, 0x01]); // Bold on
            escpos.extend_from_slice(trimmed[2..trimmed.len()-2].trim().as_bytes());
            escpos.extend_from_slice(&[0x1B, 0x45, 0x00]); // Bold off
        } else if trimmed == "---" {
            // Separator line
            escpos.extend_from_slice(&[0x2D, 0x2D, 0x2D, 0x2D, 0x2D, 0x2D, 0x2D, 0x2D, 0x2D, 0x2D]);
        } else {
            // Normal text
            escpos.extend_from_slice(trimmed.as_bytes());
        }
        
        escpos.push(0x0A); // Line feed
    }
    
    // Cut paper
    escpos.extend_from_slice(&[0x1D, 0x56, 0x42, 0x00]); // Partial cut
    
    Ok(escpos)
}
