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
    #[cfg(target_os = "windows")]
    {
        // Windows: Use lp command (if available) or direct printer port access
        use std::fs;
        use std::io::Write;
        use std::os::windows::process::CommandExt;
        
        // Try using lp command first (if CUPS is installed or Windows Print Spooler API)
        // Otherwise, try direct file copy to printer port
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        // Method 1: Try using PowerShell with raw printer port
        let temp_file = std::env::temp_dir().join(format!("print_{}.tmp", uuid::Uuid::new_v4().as_simple()));
        fs::write(&temp_file, data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        // Use PowerShell to send raw data to printer via printer port
        let file_path_escaped = temp_file.to_string_lossy().replace('\\', "\\\\").replace('\'', "''");
        let ps_script = format!(
            "$printer = Get-Printer -Name '{}' -ErrorAction Stop; $portName = $printer.PortName; $data = [System.IO.File]::ReadAllBytes('{}'); try {{ $port = New-Object System.IO.FileStream($portName, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Write); $port.Write($data, 0, $data.Length); $port.Close(); Write-Output 'Print successful' }} catch {{ Write-Error $_.Exception.Message }}",
            printer_name,
            file_path_escaped
        );
        
        let output = Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-Command",
                &ps_script
            ])
            .output();
        
        let _ = fs::remove_file(&temp_file);
        
        match output {
            Ok(result) if result.status.success() => {
                return Ok(());
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                // Fallback: Try using copy command to printer share
                return try_copy_to_printer_share(printer_name, data);
            }
            Err(_) => {
                // Fallback: Try using copy command to printer share
                return try_copy_to_printer_share(printer_name, data);
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    fn try_copy_to_printer_share(printer_name: &str, data: &[u8]) -> Result<(), String> {
        use std::fs;
        use std::os::windows::process::CommandExt;
        
        let temp_file = std::env::temp_dir().join(format!("print_{}.tmp", uuid::Uuid::new_v4().as_simple()));
        fs::write(&temp_file, data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        // Try copying to printer share (\\localhost\printer_name)
        let output = Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&[
                "/c",
                "copy",
                "/B",
                &temp_file.to_string_lossy(),
                &format!("\\\\localhost\\{}", printer_name)
            ])
            .output()
            .map_err(|e| format!("Failed to print: {}", e))?;
        
        let _ = fs::remove_file(&temp_file);
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Print failed: {}", stderr));
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    fn try_copy_to_printer_share(_printer_name: &str, _data: &[u8]) -> Result<(), String> {
        Err("Not available on this platform".to_string())
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
