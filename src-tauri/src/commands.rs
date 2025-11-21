use crate::printer::*;
use serde::{Deserialize, Serialize};
use tauri::{command, State};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StationConfig {
    pub websocket_url: String,
    pub station_name: String,
    pub watch_folder: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrinterConfig {
    pub id: String,
    pub name: String,
    pub printer_type: String, // "escpos", "zpl", "office", "pdf"
    pub system_printer: String,
}

// Removed unused type aliases

#[command]
pub fn get_system_printers() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use PowerShell to list printers
        use std::process::Command;
        
        // Try PowerShell method first
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let output = Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-Command",
                "Get-Printer | Select-Object -ExpandProperty Name | ConvertTo-Json -AsArray"
            ])
            .output();
        
        match output {
            Ok(result) => {
                if !result.status.success() {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    #[cfg(debug_assertions)]
                    eprintln!("PowerShell error: {}", stderr);
                    
                    // Fallback: Try wmic command
                    return get_printers_wmic();
                }
                
                let stdout = String::from_utf8_lossy(&result.stdout);
                #[cfg(debug_assertions)]
                eprintln!("PowerShell output: {}", stdout);
                
                match serde_json::from_str::<Vec<String>>(&stdout) {
                    Ok(printers) => {
                        #[cfg(debug_assertions)]
                        eprintln!("Successfully parsed {} printers", printers.len());
                        Ok(printers)
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("Failed to parse JSON: {}. Output: {}", e, stdout);
                        // Fallback: Try parsing line by line if it's not JSON
                        let printers: Vec<String> = stdout
                            .lines()
                            .filter_map(|line| {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() && !trimmed.starts_with('[') && !trimmed.starts_with(']') {
                                    Some(trimmed.trim_matches('"').to_string())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        
                        if !printers.is_empty() {
                            Ok(printers)
                        } else {
                            // Last resort: try wmic
                            get_printers_wmic()
                        }
                    }
                }
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("Failed to execute PowerShell: {}", e);
                // Fallback: Try wmic command
                get_printers_wmic()
            }
        }
    }
    
    
    #[cfg(target_os = "macos")]
    {
        // macOS: Use lpstat to list printers
        use std::process::Command;
        let output = Command::new("lpstat")
            .args(&["-p"])
            .output()
            .map_err(|e| format!("Failed to get printers: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let printers: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                if line.starts_with("printer ") {
                    line.split_whitespace().nth(1).map(String::from)
                } else {
                    None
                }
            })
            .collect();
        Ok(printers)
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok(vec![])
    }
}

#[cfg(target_os = "windows")]
fn get_printers_wmic() -> Result<Vec<String>, String> {
    use std::process::Command;
    
    // Fallback: Use wmic to get printers
    let output = Command::new("wmic")
        .args(&["printer", "get", "name", "/value"])
        .output()
        .map_err(|e| format!("Failed to execute wmic: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let printers: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            if line.starts_with("Name=") {
                let name = line.strip_prefix("Name=")?.trim();
                if !name.is_empty() {
                    Some(name.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    
        if printers.is_empty() {
            Err("No printers found on the system".to_string())
        } else {
            #[cfg(debug_assertions)]
            eprintln!("Found {} printers using wmic", printers.len());
            Ok(printers)
        }
}

#[cfg(not(target_os = "windows"))]
fn get_printers_wmic() -> Result<Vec<String>, String> {
    Err("wmic only available on Windows".to_string())
}

#[command]
pub fn get_station_config(
    app_handle: tauri::AppHandle,
) -> Result<StationConfig, String> {
    let config_path = app_handle
        .path_resolver()
        .app_config_dir()
        .ok_or("Config directory not found")?
        .join("station_config.json");
    
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let config: StationConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        Ok(config)
    } else {
        Ok(StationConfig {
            websocket_url: "ws://localhost:8080".to_string(),
            station_name: "Station-1".to_string(),
            watch_folder: "".to_string(),
        })
    }
}

#[command]
pub fn save_station_config(
    config: StationConfig,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let config_path = app_handle
        .path_resolver()
        .app_config_dir()
        .ok_or("Config directory not found")?
        .join("station_config.json");
    
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    Ok(())
}

#[command]
pub fn get_printers(
    app_handle: tauri::AppHandle,
) -> Result<Vec<PrinterConfig>, String> {
    let printers_path = app_handle
        .path_resolver()
        .app_config_dir()
        .ok_or("Config directory not found")?
        .join("printers.json");
    
    if printers_path.exists() {
        let content = std::fs::read_to_string(&printers_path)
            .map_err(|e| format!("Failed to read printers: {}", e))?;
        let printers: Vec<PrinterConfig> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse printers: {}", e))?;
        Ok(printers)
    } else {
        Ok(vec![])
    }
}

#[command]
pub fn save_printer(
    printer: PrinterConfig,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let printers_path = app_handle
        .path_resolver()
        .app_config_dir()
        .ok_or("Config directory not found")?
        .join("printers.json");
    
    let mut printers = get_printers(app_handle.clone())?;
    
    if let Some(pos) = printers.iter().position(|p| p.id == printer.id) {
        printers[pos] = printer;
    } else {
        printers.push(printer);
    }
    
    let content = serde_json::to_string_pretty(&printers)
        .map_err(|e| format!("Failed to serialize printers: {}", e))?;
    
    std::fs::write(&printers_path, content)
        .map_err(|e| format!("Failed to write printers: {}", e))?;
    
    Ok(())
}

#[command]
pub fn delete_printer(
    printer_id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let printers_path = app_handle
        .path_resolver()
        .app_config_dir()
        .ok_or("Config directory not found")?
        .join("printers.json");
    
    let mut printers = get_printers(app_handle.clone())?;
    printers.retain(|p| p.id != printer_id);
    
    let content = serde_json::to_string_pretty(&printers)
        .map_err(|e| format!("Failed to serialize printers: {}", e))?;
    
    std::fs::write(&printers_path, content)
        .map_err(|e| format!("Failed to write printers: {}", e))?;
    
    Ok(())
}

#[command]
pub fn start_file_watcher(
    folder_path: String,
    app_handle: tauri::AppHandle,
    watcher: State<'_, crate::watcher::FileWatcher>,
) -> Result<(), String> {
    watcher.start(folder_path, app_handle)
        .map_err(|e| format!("Failed to start watcher: {}", e.to_string()))?;
    Ok(())
}

#[command]
pub fn stop_file_watcher(
    watcher: State<'_, crate::watcher::FileWatcher>,
) -> Result<(), String> {
    watcher.stop()
        .map_err(|e| format!("Failed to stop watcher: {}", e.to_string()))?;
    Ok(())
}

#[command]
pub fn test_print_escpos(
    printer_name: String,
    receiptline_markdown: String,
) -> Result<(), String> {
    print_escpos_receiptline(&printer_name, &receiptline_markdown)
}

#[command]
pub fn test_print_zpl(
    printer_name: String,
    zpl_code: String,
) -> Result<(), String> {
    print_zpl(&printer_name, &zpl_code)
}

#[command]
pub fn test_print_office(
    printer_name: String,
    pdf_base64: String,
) -> Result<(), String> {
    print_office_pdf(&printer_name, &pdf_base64)
}

#[command]
pub fn test_print_pdf(
    pdf_base64: String,
) -> Result<(), String> {
    print_pdf(&pdf_base64)
}
