use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub struct FileWatcher {
    inner: Mutex<Option<notify::RecommendedWatcher>>,
    processed_files: Arc<Mutex<HashSet<PathBuf>>>,
    pending_files: Arc<Mutex<HashSet<PathBuf>>>,
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self {
            inner: Mutex::new(None),
            processed_files: Arc::new(Mutex::new(HashSet::new())),
            pending_files: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl FileWatcher {
    pub fn start(&self, folder_path: String, app_handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        self.stop()?;
        
        // Clear processed files when restarting
        self.processed_files.lock().unwrap().clear();
        self.pending_files.lock().unwrap().clear();
        
        let path = Path::new(&folder_path);
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        
        let app_handle_clone = app_handle.clone();
        let processed_files_clone = Arc::clone(&self.processed_files);
        let pending_files_clone = Arc::clone(&self.pending_files);
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Only handle Create events to avoid duplicate processing
                    // On macOS, files are copied atomically, so Create is the reliable event
                    if let EventKind::Create(_) = event.kind {
                        let spl_files: Vec<PathBuf> = event.paths.iter()
                            .filter_map(|file_path| {
                                if let Some(ext) = file_path.extension() {
                                    let ext_str = ext.to_string_lossy().to_lowercase();
                                    if ext_str == "spl" {
                                        Some(file_path.clone())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();
                        
                        if !spl_files.is_empty() {
                            #[cfg(debug_assertions)]
                            eprintln!("Event received with {} .spl file(s)", spl_files.len());
                            
                            let total_files = spl_files.len();
                            for (idx, file_path_buf) in spl_files.into_iter().enumerate() {
                                // Check if we've already processed or are processing this file
                                let already_processed = {
                                    let processed = processed_files_clone.lock().unwrap();
                                    processed.contains(&file_path_buf)
                                };
                                
                                let already_pending = {
                                    let pending = pending_files_clone.lock().unwrap();
                                    pending.contains(&file_path_buf)
                                };
                                
                                if !already_processed && !already_pending {
                                    let file_idx = idx; // Capture index for use in closure
                                    #[cfg(debug_assertions)]
                                    eprintln!("[{}/{}] New file detected: {:?}", file_idx + 1, total_files, file_path_buf);
                                    
                                    // Mark as pending immediately
                                    {
                                        let mut pending = pending_files_clone.lock().unwrap();
                                        pending.insert(file_path_buf.clone());
                                    }
                                    
                                    // Spawn async handling with debounce delay
                                    let app_handle_spawn = app_handle_clone.clone();
                                    let processed_files_spawn = Arc::clone(&processed_files_clone);
                                    let pending_files_spawn = Arc::clone(&pending_files_clone);
                                    let file_name = file_path_buf.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    let file_path_for_log = file_path_buf.clone();
                                    let total_files_captured = total_files; // Capture for closure
                                    
                                    std::thread::spawn(move || {
                                        // Wait for file to be fully copied (macOS paste operation delay)
                                        // Use a small staggered delay per file to avoid conflicts
                                        let delay = 1000u64 + ((file_idx as u64) * 100);
                                        #[cfg(debug_assertions)]
                                        eprintln!("Waiting {}ms before processing file {}/{}: {}", delay, file_idx + 1, total_files_captured, file_name);
                                        std::thread::sleep(Duration::from_millis(delay));
                                        
                                        #[cfg(debug_assertions)]
                                        eprintln!("Starting to process file {}/{}: {}", file_idx + 1, total_files_captured, file_name);
                                        if let Err(e) = handle_file_created(&file_path_for_log, &app_handle_spawn) {
                                            #[cfg(debug_assertions)]
                                            eprintln!("Error handling file {:?}: {}", file_path_for_log, e);
                                        } else {
                                            #[cfg(debug_assertions)]
                                            eprintln!("Successfully processed file {}/{}: {}", file_idx + 1, total_files_captured, file_name);
                                        }
                                        
                                        // Mark as processed
                                        {
                                            let mut processed = processed_files_spawn.lock().unwrap();
                                            processed.insert(file_path_for_log.clone());
                                        }
                                        
                                        // Remove from pending
                                        {
                                            let mut pending = pending_files_spawn.lock().unwrap();
                                            pending.remove(&file_path_for_log);
                                        }
                                    });
                                    } else {
                                        #[cfg(debug_assertions)]
                                        eprintln!("File already processed or pending: {:?}", file_path_buf);
                                    }
                            }
                        }
                    }
                }
                Err(e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Watcher error: {:?}", e);
                }
            }
        })?;
        
        watcher.watch(path, RecursiveMode::NonRecursive)?;
        
        *self.inner.lock().unwrap() = Some(watcher);
        
        Ok(())
    }

    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(_watcher) = self.inner.lock().unwrap().take() {
            // Watcher will be dropped here
        }
        Ok(())
    }
}

fn handle_file_created(file_path: &Path, app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use std::thread;
    
    let path_str = file_path.to_str().ok_or("Invalid path")?;
    #[cfg(debug_assertions)]
    eprintln!("Attempting to read file: {}", path_str);
    
    // Retry reading the file in case it's still being written
    let mut retries = 30;
    let mut last_size = 0u64;
    let mut stable_count = 0;
    
    let content = loop {
        // Check if file exists and log the attempt
        if !file_path.exists() {
            #[cfg(debug_assertions)]
            eprintln!("File does not exist yet, retries remaining: {}", retries);
            if retries > 0 {
                retries -= 1;
                thread::sleep(Duration::from_millis(200));
                continue;
            } else {
                #[cfg(debug_assertions)]
                eprintln!("File still does not exist after all retries: {}", path_str);
                return Err(format!("File does not exist after retries: {}", path_str).into());
            }
        }
        
        // Try to read the file
        match std::fs::read(file_path) {
            Ok(data) => {
                // Check if file size is stable (file is complete)
                if let Ok(metadata) = std::fs::metadata(file_path) {
                    let current_size = metadata.len();
                    #[cfg(debug_assertions)]
                    eprintln!("File size: {} bytes (attempt {})", current_size, 31 - retries);
                    
                    if current_size == last_size && current_size > 0 {
                        stable_count += 1;
                        // File size is stable for 2 checks, assume it's complete
                        if stable_count >= 2 {
                            #[cfg(debug_assertions)]
                            eprintln!("File size stable, reading content");
                            break data;
                        }
                    } else {
                        stable_count = 0;
                        last_size = current_size;
                    }
                }
                
                if retries > 0 {
                    retries -= 1;
                    thread::sleep(Duration::from_millis(200));
                } else {
                    // Use what we have if no more retries
                    #[cfg(debug_assertions)]
                    eprintln!("No more retries, using current content");
                    break data;
                }
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("Error reading file (retries: {}): {}", retries, e);
                if retries > 0 && e.kind() == std::io::ErrorKind::NotFound {
                    retries -= 1;
                    thread::sleep(Duration::from_millis(200));
                    continue;
                } else {
                    return Err(format!("Failed to read file {}: {}", path_str, e).into());
                }
            }
        }
    };
    
    #[cfg(debug_assertions)]
    eprintln!("Successfully read {} bytes from file: {}", content.len(), path_str);
    
    // Emit event to frontend
    use base64::{Engine as _, engine::general_purpose};
    app_handle.emit_all("file-detected", serde_json::json!({
        "path": path_str,
        "content": general_purpose::STANDARD.encode(&content)
    }))?;
    
    #[cfg(debug_assertions)]
    eprintln!("Event emitted to frontend");
    
    // Delete the file after reading (only if it still exists)
    if file_path.exists() {
        match std::fs::remove_file(file_path) {
            Ok(_) => {
                #[cfg(debug_assertions)]
                eprintln!("File deleted successfully: {}", path_str);
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Could not delete file {}: {}", path_str, e);
            }
        }
    } else {
        #[cfg(debug_assertions)]
        eprintln!("File no longer exists, skipping deletion: {}", path_str);
    }
    
    Ok(())
}
