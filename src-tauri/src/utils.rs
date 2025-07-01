use std::env;
use std::path::Path;
use std::process::Command;

/// Get OS-specific search paths for Pandoc
pub fn get_search_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // First try PATH
    if let Ok(path_pandoc) = get_path_pandoc_internal() {
        paths.push(path_pandoc);
    }

    // Platform-specific common paths
    let common_paths = if cfg!(target_os = "windows") {
        get_windows_common_paths()
    } else if cfg!(target_os = "macos") {
        get_macos_common_paths()
    } else {
        get_linux_common_paths()
    };

    // Add common paths, avoiding duplicates
    for path in common_paths {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }

    paths
}

/// Windows common installation paths
fn get_windows_common_paths() -> Vec<String> {
    let mut paths = Vec::new();

    if let Ok(userprofile) = env::var("USERPROFILE") {
        paths.push(format!(
            "{}\\AppData\\Roaming\\pandoc\\pandoc.exe",
            userprofile
        ));
        paths.push(format!(
            "{}\\AppData\\Local\\pandoc\\pandoc.exe",
            userprofile
        ));
        paths.push(format!(
            "{}\\AppData\\Local\\Pandoc\\pandoc.exe",
            userprofile
        ));
        paths.push(format!(
            "{}\\scoop\\apps\\pandoc\\current\\pandoc.exe",
            userprofile
        ));
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

/// macOS common installation paths
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

/// Linux common installation paths
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

/// Find pandoc in PATH
fn get_path_pandoc_internal() -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        create_hidden_command("where").arg("pandoc").output()
    } else {
        create_hidden_command("which").arg("pandoc").output()
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

/// Validate if a path contains a working pandoc executable
pub fn validate_pandoc_executable(path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }

    create_hidden_command(path)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get current platform's Pandoc download asset patterns (in order of preference)
pub fn get_pandoc_asset_patterns() -> Vec<&'static str> {
    if cfg!(target_os = "windows") {
        vec![
            "windows-x86_64.zip",   // Prefer portable version
            "windows-x86_64.msi",   // Fallback to installer (not supported yet)
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "macOS.zip",           // Prefer portable version
            "macOS.pkg",           // Fallback to installer (not supported yet)
        ]
    } else {
        vec![
            "linux-amd64.tar.gz", // Linux binary
        ]
    }
}

/// Get current platform's primary Pandoc download asset pattern (for backward compatibility)
#[allow(dead_code)]
pub fn get_pandoc_asset_pattern() -> &'static str {
    get_pandoc_asset_patterns()[0]
}

/// Format file size in human readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Create a hidden command to avoid PowerShell popup on Windows
pub fn create_hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // Use CREATE_NO_WINDOW flag to hide the console window
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    cmd
}
