use std::process::Command;
use std::path::Path;
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PandocInfo {
    pub version: String,
    pub path: String,
    pub is_working: bool,
    pub supported_input_formats: Vec<String>,
    pub supported_output_formats: Vec<String>,
    pub detected_paths: Vec<String>,
    pub search_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PandocConfig {
    pub custom_path: Option<String>,
    pub use_custom_path: bool,
    pub last_detected_version: String,
}

impl Default for PandocConfig {
    fn default() -> Self {
        Self {
            custom_path: None,
            use_custom_path: false,
            last_detected_version: String::new(),
        }
    }
}

// Get comprehensive pandoc information
#[tauri::command]
async fn get_pandoc_info(custom_path: Option<String>) -> Result<PandocInfo, String> {
    let search_paths = get_search_paths();
    let detected_paths = find_all_pandoc_paths();
    
    let pandoc_cmd = if let Some(custom_path) = custom_path {
        custom_path
    } else if !detected_paths.is_empty() {
        detected_paths[0].clone()
    } else {
        return Err(format!(
            "Pandoc not found. Searched {} locations including:\n{}",
            search_paths.len(),
            search_paths.iter().take(5).map(|p| format!("- {}", p)).collect::<Vec<_>>().join("\n")
        ));
    };
    
    // Check if pandoc exists and get version
    let version_output = Command::new(&pandoc_cmd)
        .arg("--version")
        .output();
    
    match version_output {
        Ok(output) if output.status.success() => {
            let version_text = String::from_utf8(output.stdout)
                .map_err(|_| "Failed to read pandoc version output".to_string())?;
            
            let version = version_text
                .lines()
                .next()
                .unwrap_or("Unknown version")
                .to_string();
            
            // Get supported formats
            let (input_formats, output_formats) = get_supported_formats(&pandoc_cmd)?;
            
            Ok(PandocInfo {
                version,
                path: pandoc_cmd,
                is_working: true,
                supported_input_formats: input_formats,
                supported_output_formats: output_formats,
                detected_paths: detected_paths,
                search_paths: search_paths,
            })
        }
        Ok(_) => Err(format!("Pandoc at '{}' failed to execute", pandoc_cmd)),
        Err(e) => Err(format!("Failed to execute pandoc at '{}': {}", pandoc_cmd, e)),
    }
}

// Enhanced pandoc path detection with common installation paths
#[tauri::command]
async fn get_pandoc_path() -> Result<String, String> {
    let detected_paths = find_all_pandoc_paths();
    
    if detected_paths.is_empty() {
        Err("Pandoc not found in any common locations. Searched paths will be shown in detailed info.".to_string())
    } else {
        Ok(detected_paths[0].clone())
    }
}

// Find all possible pandoc installations
fn find_all_pandoc_paths() -> Vec<String> {
    let mut valid_paths = Vec::new();
    let search_paths = get_search_paths();
    
    for path in search_paths {
        if validate_pandoc_executable(&path) {
            valid_paths.push(path);
        }
    }
    
    valid_paths
}

// Get comprehensive search paths based on OS
fn get_search_paths() -> Vec<String> {
    let mut paths = Vec::new();
    
    // First try PATH
    if let Ok(path_pandoc) = get_path_pandoc_internal() {
        paths.push(path_pandoc);
    }
    
    // Platform-specific common paths
    if cfg!(target_os = "windows") {
        paths.extend(get_windows_common_paths());
    } else if cfg!(target_os = "macos") {
        paths.extend(get_macos_common_paths());
    } else {
        paths.extend(get_linux_common_paths());
    }
    
    paths
}

// Windows common installation paths
fn get_windows_common_paths() -> Vec<String> {
    let mut paths = Vec::new();
    
    if let Ok(userprofile) = env::var("USERPROFILE") {
        paths.push(format!("{}\\AppData\\Roaming\\pandoc\\pandoc.exe", userprofile));
        paths.push(format!("{}\\AppData\\Local\\pandoc\\pandoc.exe", userprofile));
        paths.push(format!("{}\\AppData\\Local\\Pandoc\\pandoc.exe", userprofile));
        paths.push(format!("{}\\scoop\\apps\\pandoc\\current\\pandoc.exe", userprofile));
    }
    
    paths.push("C:\\Program Files\\Pandoc\\pandoc.exe".to_string());
    paths.push("C:\\Program Files (x86)\\Pandoc\\pandoc.exe".to_string());
    
    if let Ok(chocolatey) = env::var("ChocolateyInstall") {
        paths.push(format!("{}\\bin\\pandoc.exe", chocolatey));
    }
    
    // Conda environments
    if let Ok(conda_prefix) = env::var("CONDA_PREFIX") {
        paths.push(format!("{}\\Scripts\\pandoc.exe", conda_prefix));
        paths.push(format!("{}\\bin\\pandoc.exe", conda_prefix));
    }
    
    paths
}

// macOS common installation paths  
fn get_macos_common_paths() -> Vec<String> {
    let mut paths = Vec::new();
    
    // Homebrew paths
    paths.push("/usr/local/bin/pandoc".to_string());
    paths.push("/opt/homebrew/bin/pandoc".to_string());
    
    // User installations
    if let Ok(home) = env::var("HOME") {
        paths.push(format!("{}/Library/Haskell/bin/pandoc", home));
        paths.push(format!("{}/.local/bin/pandoc", home));
        paths.push(format!("{}/.cabal/bin/pandoc", home));
    }
    
    // System paths
    paths.push("/usr/bin/pandoc".to_string());
    paths.push("/usr/local/bin/pandoc".to_string());
    
    paths
}

// Linux common installation paths
fn get_linux_common_paths() -> Vec<String> {
    let mut paths = Vec::new();
    
    // System paths
    paths.push("/usr/bin/pandoc".to_string());
    paths.push("/usr/local/bin/pandoc".to_string());
    
    // User paths
    if let Ok(home) = env::var("HOME") {
        paths.push(format!("{}/.local/bin/pandoc", home));
        paths.push(format!("{}/.cabal/bin/pandoc", home));
    }
    
    paths
}

// Try to find pandoc in PATH (internal helper)
fn get_path_pandoc_internal() -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("where").arg("pandoc").output()
    } else {
        Command::new("which").arg("pandoc").output()
    };
    
    match output {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8(output.stdout)
                .map_err(|_| "Failed to read path output".to_string())?;
            Ok(path.lines().next().unwrap_or("").trim().to_string())
        }
        _ => Err("Not found in PATH".to_string()),
    }
}

// Validate if a path contains a working pandoc executable
fn validate_pandoc_executable(path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }
    
    Command::new(path)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Check if a custom path is valid
#[tauri::command]
async fn validate_pandoc_path(path: String) -> Result<bool, String> {
    if !Path::new(&path).exists() {
        return Ok(false);
    }
    
    let output = Command::new(&path)
        .arg("--version")
        .output();
    
    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

// Get supported input and output formats with improved error handling
fn get_supported_formats(pandoc_cmd: &str) -> Result<(Vec<String>, Vec<String>), String> {
    // Common fallback formats
    let fallback_input_formats = vec![
        "markdown".to_string(), "html".to_string(), "latex".to_string(),
        "rst".to_string(), "docx".to_string(), "epub".to_string(),
        "org".to_string(), "textile".to_string(), "commonmark".to_string(),
        "gfm".to_string(), "json".to_string(),
    ];
    
    let fallback_output_formats = vec![
        "html".to_string(), "latex".to_string(), "pdf".to_string(),
        "docx".to_string(), "epub".to_string(), "rst".to_string(),
        "markdown".to_string(), "plain".to_string(), "json".to_string(),
        "commonmark".to_string(), "gfm".to_string(),
    ];
    
    // Try to get input formats, use fallback if fails
    let input_formats = Command::new(pandoc_cmd)
        .args(&["--list-input-formats"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|content| {
                        content
                            .lines()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
            } else {
                None
            }
        })
        .unwrap_or_else(|| fallback_input_formats.clone());
    
    // Try to get output formats, use fallback if fails
    let output_formats = Command::new(pandoc_cmd)
        .args(&["--list-output-formats"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|content| {
                        content
                            .lines()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
            } else {
                None
            }
        })
        .unwrap_or_else(|| fallback_output_formats.clone());
    
    Ok((input_formats, output_formats))
}

// Enhanced pandoc conversion with optional input format (auto-detection)
#[tauri::command]
async fn convert_with_pandoc(
    input_file: String,
    output_file: String,
    input_format: Option<String>,
    output_format: String,
    custom_pandoc_path: Option<String>,
) -> Result<String, String> {
    // Determine which pandoc path to use
    let pandoc_cmd = if let Some(custom_path) = custom_pandoc_path {
        custom_path
    } else {
        // Try to find pandoc automatically
        let detected_paths = find_all_pandoc_paths();
        if detected_paths.is_empty() {
            return Err("Pandoc not found. Please check your installation or specify a custom path in settings.".to_string());
        }
        detected_paths[0].clone()
    };
    
    // Validate output format only (input format is optional for auto-detection)
    let (_, supported_outputs) = get_supported_formats(&pandoc_cmd)
        .unwrap_or_else(|_| {
            // If format detection fails, use fallback formats
            (vec![], vec![
                "html".to_string(), "latex".to_string(), "pdf".to_string(),
                "docx".to_string(), "epub".to_string(), "rst".to_string(),
                "markdown".to_string(), "plain".to_string(),
            ])
        });
    
    if !supported_outputs.contains(&output_format) {
        return Err(format!("Output format '{}' is not supported by this Pandoc installation", output_format));
    }
    
    // Build command arguments
    let mut args = Vec::new();
    
    // Add input format only if specified (otherwise let Pandoc auto-detect)
    if let Some(input_fmt) = input_format {
        if !input_fmt.is_empty() && input_fmt != "auto" {
            args.push("-f".to_string());
            args.push(input_fmt);
        }
    }
    
    // Add output format
    args.push("-t".to_string());
    args.push(output_format.clone());
    
    // Add input and output files
    args.push(input_file.clone());
    args.push("-o".to_string());
    args.push(output_file.clone());
    
    // Execute conversion
    let output = Command::new(&pandoc_cmd)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute pandoc at '{}': {}", pandoc_cmd, e))?;
    
    if output.status.success() {
        Ok(format!("Successfully converted {} to {}", input_file, output_file))
    } else {
        let error_msg = String::from_utf8(output.stderr)
            .unwrap_or_else(|_| "Unknown pandoc error".to_string());
        Err(format!("Pandoc conversion failed: {}", error_msg.trim()))
    }
}

// Legacy function for compatibility
#[tauri::command]
async fn check_pandoc_version() -> Result<String, String> {
    let info = get_pandoc_info(None).await?;
    Ok(info.version)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_pandoc_info,
            get_pandoc_path,
            validate_pandoc_path,
            convert_with_pandoc,
            check_pandoc_version
        ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
