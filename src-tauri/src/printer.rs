use std::process::Command;
use std::io::Write;

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
        // Windows: Use PowerShell to print PDF
        use std::fs;
        let temp_file = std::env::temp_dir().join(format!("print_{}.pdf", uuid::Uuid::new_v4().as_simple()));
        fs::write(&temp_file, &pdf_data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!("Start-Process -FilePath '{}' -Verb Print -WindowStyle Hidden", 
                    temp_file.to_str().unwrap())
            ])
            .output()
            .map_err(|e| format!("Failed to print: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Print failed: {}", String::from_utf8_lossy(&output.stderr)));
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
        Command::new("powershell")
            .args(&[
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
    #[cfg(target_os = "windows")]
    {
        // Windows: Use copy command to send raw data to printer
        use std::fs;
        let temp_file = std::env::temp_dir().join(format!("print_{}.tmp", uuid::Uuid::new_v4().as_simple()));
        fs::write(&temp_file, data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!("Copy-Item '{}' '\\\\localhost\\{}'", 
                    temp_file.to_str().unwrap(), printer_name)
            ])
            .output()
            .map_err(|e| format!("Failed to print: {}", e))?;
        
        let _ = fs::remove_file(&temp_file);
        
        if !output.status.success() {
            return Err(format!("Print failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Stdio;
        let mut child = Command::new("lp")
            .args(&["-d", printer_name, "-o", "raw"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start lp: {}", e))?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(data)
                .map_err(|e| format!("Failed to write to printer: {}", e))?;
        }
        
        let output = child.wait_with_output()
            .map_err(|e| format!("Failed to wait for print: {}", e))?;
        
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
