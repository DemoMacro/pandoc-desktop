use crate::types::{GithubAsset, GithubRelease, PandocInfo, VersionInfo};
use crate::utils::format_file_size;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;

const UNGH_API_BASE: &str = "https://ungh.cc/repos";
const PANDOC_REPO: &str = "jgm/pandoc";
const TYPST_REPO: &str = "typst/typst";

/// Mirror URLs for downloading (in order of preference)
const DOWNLOAD_MIRRORS: &[&str] = &[
    "https://hub.gitmirror.com/", // 7ed.net mirror
    "https://gh.ddlc.top/",       // GitHub proxy
    "",                           // Original GitHub (empty prefix)
];

/// Download types supported by the unified download system
#[derive(Debug, Clone)]
pub enum DownloadType {
    Pandoc,
    Typst,
}

/// Configuration for downloads
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub target_os: String,
    pub target_arch: String,
    pub use_mirrors: bool,
}

impl DownloadConfig {
    /// Create download config for current platform (for runtime)
    pub fn current_platform() -> Self {
        Self {
            target_os: std::env::consts::OS.to_string(),
            target_arch: std::env::consts::ARCH.to_string(),
            use_mirrors: true,
        }
    }
}

/// Pandoc source types with priority order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PandocSource {
    /// Custom path set by user (highest priority)
    Custom(PathBuf),
    /// Managed pandoc (bundled with app or downloaded portable)
    Managed,
    /// System-detected pandoc (lowest priority)
    System(PathBuf),
}

/// Complete pandoc information including source and details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PandocManager {
    pub source: PandocSource,
    pub info: Option<PandocInfo>,
    pub available: bool,
}

impl PandocManager {
    /// Create a new PandocManager
    pub fn new(source: PandocSource) -> Self {
        Self {
            source,
            info: None,
            available: false,
        }
    }

    /// Get the executable path for this pandoc source
    pub fn get_executable_path(&self, app_handle: &AppHandle) -> Option<PathBuf> {
        match &self.source {
            PandocSource::Custom(path) => Some(path.clone()),
            PandocSource::Managed => get_managed_pandoc_path(app_handle),
            PandocSource::System(path) => Some(path.clone()),
        }
    }

    /// Validate and update pandoc information
    pub async fn validate(&mut self, app_handle: &AppHandle) -> Result<(), String> {
        if let Some(path) = self.get_executable_path(app_handle) {
            match validate_pandoc_executable(&path).await {
                Ok(info) => {
                    self.info = Some(info);
                    self.available = true;
                    Ok(())
                }
                Err(e) => {
                    self.info = None;
                    self.available = false;
                    Err(e)
                }
            }
        } else {
            self.info = None;
            self.available = false;
            Err("Failed to get executable path".to_string())
        }
    }

    /// Check if this pandoc source is available
    #[allow(dead_code)]
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Get pandoc info if available
    pub fn get_info(&self) -> Option<&PandocInfo> {
        self.info.as_ref()
    }
}

/// Get managed pandoc path from multiple locations (unified bundled/portable logic)
fn get_managed_pandoc_path(app_handle: &AppHandle) -> Option<PathBuf> {
    let pandoc_exe = if cfg!(windows) {
        "pandoc.exe"
    } else {
        "pandoc"
    };

    // Priority 1: Check resources directory (bundled with app during build)
    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        let resource_pandoc_dir = resource_dir.join("pandoc");
        if let Some(path) = find_pandoc_in_directory(&resource_pandoc_dir, &pandoc_exe) {
            return Some(path);
        }
    }

    // Priority 2: Check app data directory (portable downloaded by user)
    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        let portable_pandoc_dir = app_data_dir.join("pandoc-portable");
        if let Some(path) = find_pandoc_in_directory(&portable_pandoc_dir, &pandoc_exe) {
            return Some(path);
        }
    }

    None
}

/// Find pandoc executable in a directory (with common subdirectory patterns)
fn find_pandoc_in_directory(base_dir: &PathBuf, exe_name: &str) -> Option<PathBuf> {
    if !base_dir.exists() {
        return None;
    }

    // Check common locations
    let candidate_paths = vec![
        base_dir.join(exe_name),             // Direct path
        base_dir.join("bin").join(exe_name), // In bin subdirectory
    ];

    for path in candidate_paths {
        if path.exists() {
            return Some(path);
        }
    }

    // Check for pandoc in any subdirectory (version directories are common)
    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let sub_paths = vec![
                    entry.path().join(exe_name),
                    entry.path().join("bin").join(exe_name),
                ];

                for path in sub_paths {
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

/// Validate pandoc executable and get its info
async fn validate_pandoc_executable(path: &PathBuf) -> Result<PandocInfo, String> {
    if !path.exists() {
        return Err("Pandoc executable not found".to_string());
    }

    let output = crate::utils::create_hidden_command(&path.to_string_lossy())
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute pandoc: {}", e))?;

    if !output.status.success() {
        return Err("Pandoc failed to execute".to_string());
    }

    let version_text = String::from_utf8(output.stdout)
        .map_err(|_| "Failed to read pandoc version output".to_string())?;

    let version = extract_version_from_output(&version_text);
    let path_str = path.to_string_lossy().to_string();

    // Get supported formats using the pandoc module function
    let (input_formats, output_formats) = crate::pandoc::get_supported_formats(&path_str)
        .unwrap_or_else(|_| {
            // Fallback to basic formats if detection fails
            (
                vec![
                    "markdown".to_string(),
                    "html".to_string(),
                    "docx".to_string(),
                ],
                vec!["html".to_string(), "pdf".to_string(), "docx".to_string()],
            )
        });

    Ok(PandocInfo {
        version,
        path: path_str,
        is_working: true,
        supported_input_formats: input_formats,
        supported_output_formats: output_formats,
        detected_paths: vec![],
        search_paths: vec![],
    })
}

/// Extract version number from pandoc version output
fn extract_version_from_output(output: &str) -> String {
    let first_line = output.lines().next().unwrap_or("Unknown");

    // Handle common format: "pandoc.exe 3.7.0.2" or "pandoc 3.7.0.2"
    let parts: Vec<&str> = first_line.trim().split_whitespace().collect();

    // Look for version number in the parts (should be after program name)
    for part in &parts {
        // Skip program names
        if part.contains("pandoc") {
            continue;
        }

        // Check if this part looks like a version number (starts with digit)
        if part.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            // Extract version pattern (digits and dots)
            let mut version = String::new();
            for ch in part.chars() {
                if ch.is_ascii_digit() || ch == '.' {
                    version.push(ch);
                } else {
                    break;
                }
            }

            if !version.is_empty() && version != "." {
                return version;
            }
        }
    }

    // Fallback: return first line with pandoc names removed
    first_line
        .replace("pandoc.exe", "")
        .replace("pandoc", "")
        .trim()
        .to_string()
}

/// Extract clean version number from version string (simplified approach)
/// Examples:
/// "pandoc.exe 3.7.0.2" -> "3.7.0.2"
/// "pandoc 3.7.0.2" -> "3.7.0.2"
/// "v3.7.0.2" -> "3.7.0.2"
/// "3.7.0.2" -> "3.7.0.2"
fn normalize_version(version_str: &str) -> String {
    let text = version_str.trim();

    // Split by whitespace and look for version-like strings
    for part in text.split_whitespace() {
        // Skip common prefixes
        if part.contains("pandoc") {
            continue;
        }

        let part = part.trim_start_matches('v');

        // Check if this looks like a version number (starts with digit)
        if part.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            // Extract version pattern manually
            let mut version = String::new();
            for ch in part.chars() {
                if ch.is_ascii_digit() || ch == '.' {
                    version.push(ch);
                } else {
                    break;
                }
            }

            // Make sure we have a valid version (not just dots)
            if !version.is_empty() && version.chars().any(|c| c.is_ascii_digit()) {
                return version;
            }
        }
    }

    // Fallback: clean up the entire string
    let cleaned = text
        .replace("pandoc.exe", "")
        .replace("pandoc", "")
        .trim()
        .trim_start_matches('v')
        .trim()
        .to_string();

    if !cleaned.is_empty() {
        cleaned
    } else {
        text.to_string()
    }
}

/// Construct mirror URL based on mirror type
fn construct_mirror_url(mirror: &str, original_url: &str) -> String {
    if mirror.is_empty() {
        // Original GitHub URL
        original_url.to_string()
    } else {
        // Mirror format: mirror_prefix + original_github_url
        format!("{}{}", mirror, original_url)
    }
}

/// Get latest release information from UNGH API
#[tauri::command]
pub async fn get_latest_pandoc_release() -> Result<GithubRelease, String> {
    let url = format!("{}/{}/releases/latest", UNGH_API_BASE, PANDOC_REPO);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "API request failed with status: {}",
            response.status()
        ));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let api_response: serde_json::Value =
        serde_json::from_str(&response_text).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // UNGH wraps the release in a "release" field
    let release_data = api_response["release"].clone();

    if release_data.is_null() {
        return Err("No release data found in response".to_string());
    }

    // Parse UNGH response format
    let release = parse_ungh_release(release_data)?;
    Ok(release)
}

/// Get all available releases
#[tauri::command]
pub async fn get_pandoc_releases(limit: Option<u32>) -> Result<Vec<GithubRelease>, String> {
    let _page_limit = limit.unwrap_or(10);
    let url = format!("{}/{}/releases", UNGH_API_BASE, PANDOC_REPO);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "API request failed with status: {}",
            response.status()
        ));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let api_response: serde_json::Value =
        serde_json::from_str(&response_text).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // UNGH wraps the releases in a "releases" field
    let releases_data = api_response["releases"]
        .as_array()
        .ok_or("No releases array found in response")?;

    let mut releases = Vec::new();
    for data in releases_data {
        if let Ok(release) = parse_ungh_release(data.clone()) {
            releases.push(release);
        }
    }

    Ok(releases)
}

/// Parse UNGH API response to GithubRelease
fn parse_ungh_release(data: serde_json::Value) -> Result<GithubRelease, String> {
    let tag_name = data["tag"].as_str().unwrap_or("").to_string();

    let name = data["name"].as_str().unwrap_or(&tag_name).to_string();

    let body = data["markdown"].as_str().unwrap_or("").to_string();

    let published_at = data["publishedAt"].as_str().unwrap_or("").to_string();

    // Generate assets since UNGH doesn't provide them
    let assets = generate_github_assets(&tag_name);

    Ok(GithubRelease {
        tag_name,
        name,
        body,
        published_at,
        assets,
    })
}

/// Generate GitHub assets for a release (comprehensive list)
fn generate_github_assets(tag: &str) -> Vec<GithubAsset> {
    let base_url = format!("https://github.com/jgm/pandoc/releases/download/{}", tag);

    vec![
        // Windows assets - prefer installer over zip
        GithubAsset {
            name: format!("pandoc-{}-windows-x86_64.msi", tag),
            download_url: format!("{}/pandoc-{}-windows-x86_64.msi", base_url, tag),
            size: 0,
            content_type: "application/x-msi".to_string(),
        },
        GithubAsset {
            name: format!("pandoc-{}-windows-x86_64.zip", tag),
            download_url: format!("{}/pandoc-{}-windows-x86_64.zip", base_url, tag),
            size: 0,
            content_type: "application/zip".to_string(),
        },
        // macOS assets - prefer pkg over zip
        GithubAsset {
            name: format!("pandoc-{}-macOS.pkg", tag),
            download_url: format!("{}/pandoc-{}-macOS.pkg", base_url, tag),
            size: 0,
            content_type: "application/x-apple-diskimage".to_string(),
        },
        GithubAsset {
            name: format!("pandoc-{}-macOS.zip", tag),
            download_url: format!("{}/pandoc-{}-macOS.zip", base_url, tag),
            size: 0,
            content_type: "application/zip".to_string(),
        },
        // Linux assets
        GithubAsset {
            name: format!("pandoc-{}-linux-amd64.tar.gz", tag),
            download_url: format!("{}/pandoc-{}-linux-amd64.tar.gz", base_url, tag),
            size: 0,
            content_type: "application/gzip".to_string(),
        },
        // Source assets
        GithubAsset {
            name: format!("pandoc-{}.tar.gz", tag),
            download_url: format!("{}/pandoc-{}.tar.gz", base_url, tag),
            size: 0,
            content_type: "application/gzip".to_string(),
        },
    ]
}

/// Get version comparison info
#[tauri::command]
pub async fn get_version_info(current_version: Option<String>) -> Result<VersionInfo, String> {
    let latest_release = get_latest_pandoc_release().await?;
    let latest_version = latest_release.tag_name.clone();

    let releases = get_pandoc_releases(Some(20)).await?;
    let available_versions: Vec<String> = releases.into_iter().map(|r| r.tag_name).collect();

    // Normalize versions for comparison
    let normalized_current = current_version.as_ref().map(|v| normalize_version(v));
    let normalized_latest = normalize_version(&latest_version);

    let is_update_available = if let Some(ref current_norm) = normalized_current {
        current_norm != &normalized_latest
    } else {
        true
    };

    Ok(VersionInfo {
        current: current_version,
        latest: Some(latest_version),
        available_versions,
        is_update_available,
    })
}

/// Download pandoc for current platform with improved asset selection
#[tauri::command]
pub async fn download_pandoc(version: String, download_dir: String) -> Result<String, String> {
    let config = DownloadConfig::current_platform();
    download_tool(
        DownloadType::Pandoc,
        Some(version),
        PathBuf::from(download_dir),
        config,
    )
    .await
}

/// Download file with progress tracking
async fn download_file(url: &str, dest_path: &PathBuf) -> Result<String, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let _total_size = response.content_length().unwrap_or(0);

    // Create destination directory if it doesn't exist
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let mut file =
        std::fs::File::create(dest_path).map_err(|e| format!("Failed to create file: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    println!(
        "Downloaded {} ({})",
        dest_path.display(),
        format_file_size(bytes.len() as u64)
    );

    file.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    Ok(dest_path.to_string_lossy().to_string())
}

/// Extract downloaded archive
#[tauri::command]
pub async fn extract_pandoc_archive(
    archive_path: String,
    extract_dir: String,
) -> Result<String, String> {
    extract_archive_unified(PathBuf::from(archive_path), PathBuf::from(extract_dir)).await
}

/// Extract ZIP archive
fn extract_zip(archive_path: &PathBuf, extract_dir: &PathBuf) -> Result<String, String> {
    let file =
        std::fs::File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read file from archive: {}", e))?;

        let outpath = extract_dir.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }

            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create extracted file: {}", e))?;

            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).ok();
                // Ignore permission errors
            }
        }
    }

    Ok(extract_dir.to_string_lossy().to_string())
}

/// Extract TAR.GZ archive
fn extract_tar_gz(archive_path: &PathBuf, extract_dir: &PathBuf) -> Result<String, String> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file =
        std::fs::File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    archive
        .unpack(extract_dir)
        .map_err(|e| format!("Failed to extract TAR.GZ archive: {}", e))?;

    Ok(extract_dir.to_string_lossy().to_string())
}

/// Discover all available pandoc sources
#[tauri::command]
pub async fn discover_pandoc_sources(app_handle: AppHandle) -> Vec<PandocManager> {
    let mut sources = Vec::new();

    // 1. Check bundled pandoc (highest priority after custom)
    let bundled_source = PandocManager::new(PandocSource::Managed);
    sources.push(bundled_source);

    // 2. Discover system pandoc installations
    let system_paths = crate::utils::get_search_paths();
    for path_str in system_paths {
        let path = PathBuf::from(&path_str);
        if path.exists() && is_executable(&path) {
            let system_source = PandocManager::new(PandocSource::System(path));
            sources.push(system_source);
        }
    }

    // Validate each source
    for source in &mut sources {
        let _ = source.validate(&app_handle).await;
    }

    sources
}

/// Check if a file is executable
fn is_executable(path: &PathBuf) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            permissions.mode() & 0o111 != 0
        } else {
            false
        }
    }

    #[cfg(windows)]
    {
        // On Windows, check if it's a .exe file or if it can be executed
        path.extension().map_or(false, |ext| ext == "exe")
            || crate::utils::create_hidden_command(&path.to_string_lossy())
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
    }
}

/// Create a pandoc manager with custom path
#[allow(dead_code)]
pub fn create_custom_manager(custom_path: PathBuf) -> PandocManager {
    PandocManager::new(PandocSource::Custom(custom_path))
}

/// Create and validate a pandoc manager with custom path (Tauri command)
#[tauri::command]
pub async fn create_and_validate_custom_manager(
    custom_path: String,
    app_handle: AppHandle,
) -> Result<PandocManager, String> {
    let path = PathBuf::from(custom_path);
    let mut manager = PandocManager::new(PandocSource::Custom(path));

    manager.validate(&app_handle).await?;
    Ok(manager)
}

/// Get the best available pandoc manager
#[tauri::command]
pub async fn get_best_pandoc_manager(app_handle: AppHandle) -> Option<PandocManager> {
    let sources = discover_pandoc_sources(app_handle.clone()).await;

    for mut source in sources {
        if let Ok(()) = source.validate(&app_handle).await {
            return Some(source);
        }
    }

    None
}

/// Update bundled pandoc by downloading latest version (legacy function)
#[tauri::command]
pub async fn update_bundled_pandoc(app_handle: AppHandle) -> Result<String, String> {
    update_managed_pandoc(app_handle).await
}

/// Check if bundled pandoc needs update
#[tauri::command]
pub async fn check_bundled_pandoc_update(app_handle: AppHandle) -> Result<bool, String> {
    // Get current bundled pandoc version
    let mut bundled_manager = PandocManager::new(PandocSource::Managed);
    let current_version = if bundled_manager.validate(&app_handle).await.is_ok() {
        bundled_manager
            .get_info()
            .map(|info| info.version.clone())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "none".to_string()
    };

    // Get latest version
    let latest_release = get_latest_pandoc_release().await?;
    let latest_version = extract_clean_version(&latest_release.tag_name);
    let current_clean = extract_clean_version(&current_version);

    // Simple version comparison
    Ok(current_clean != latest_version)
}

/// Extract clean version number for comparison
fn extract_clean_version(version: &str) -> String {
    version
        .trim_start_matches('v')
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or(version)
        .to_string()
}

/// Unified download interface for different tools
pub async fn download_tool(
    download_type: DownloadType,
    version: Option<String>,
    target_dir: PathBuf,
    config: DownloadConfig,
) -> Result<String, String> {
    match download_type {
        DownloadType::Pandoc => {
            let version = if let Some(v) = version {
                v
            } else {
                let latest = get_latest_pandoc_release().await?;
                latest.tag_name
            };
            download_pandoc_internal(version, target_dir, config).await
        }
        DownloadType::Typst => {
            let version = if let Some(v) = version {
                v
            } else {
                let latest = get_latest_typst_release(&config).await?;
                latest.tag_name
            };
            download_typst_internal(version, target_dir, config).await
        }
    }
}

/// Internal pandoc download function with enhanced config support
async fn download_pandoc_internal(
    version: String,
    download_dir: PathBuf,
    config: DownloadConfig,
) -> Result<String, String> {
    let releases = get_pandoc_releases(Some(50)).await?;

    let release = releases
        .into_iter()
        .find(|r| r.tag_name == version)
        .ok_or_else(|| format!("Version {} not found", version))?;

    // Find the best matching asset using platform-specific patterns
    let asset_patterns =
        get_pandoc_asset_patterns_for_platform(&config.target_os, &config.target_arch);
    let mut selected_asset = None;

    for pattern in &asset_patterns {
        if let Some(asset) = release.assets.iter().find(|a| a.name.contains(pattern)) {
            selected_asset = Some(asset);
            break;
        }
    }

    let asset = selected_asset.ok_or_else(|| {
        let available_assets: Vec<String> = release.assets.iter().map(|a| a.name.clone()).collect();
        format!(
            "No compatible asset found for {}-{}.\nAvailable assets: {}\nLooked for patterns: {:?}",
            config.target_os,
            config.target_arch,
            available_assets.join(", "),
            asset_patterns
        )
    })?;

    let download_path = download_dir.join(&asset.name);

    // Try different mirrors if enabled
    if config.use_mirrors {
        for mirror in DOWNLOAD_MIRRORS {
            let download_url = construct_mirror_url(mirror, &asset.download_url);
            println!(
                "Trying to download {} from mirror: {}",
                asset.name,
                if mirror.is_empty() { "GitHub" } else { mirror }
            );

            match download_file(&download_url, &download_path).await {
                Ok(path) => return Ok(path),
                Err(e) => {
                    println!(
                        "Mirror {} failed: {}",
                        if mirror.is_empty() { "GitHub" } else { mirror },
                        e
                    );
                    continue;
                }
            }
        }
        Err("All download mirrors failed".to_string())
    } else {
        download_file(&asset.download_url, &download_path).await
    }
}

/// Get latest Typst release information
async fn get_latest_typst_release(config: &DownloadConfig) -> Result<GithubRelease, String> {
    // Use UNGH API for Typst (same as Pandoc)
    let url = format!("{}/{}/releases/latest", UNGH_API_BASE, TYPST_REPO);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch Typst release info: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "UNGH API request failed with status: {}",
            response.status()
        ));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let api_response: serde_json::Value =
        serde_json::from_str(&response_text).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // UNGH wraps the release in a "release" field
    let release_data = api_response["release"].clone();

    if release_data.is_null() {
        return Err("No release data found in response".to_string());
    }

    parse_ungh_release(release_data)
}

/// Internal typst download function
async fn download_typst_internal(
    _version: String,
    download_dir: PathBuf,
    config: DownloadConfig,
) -> Result<String, String> {
    let release = get_latest_typst_release(&config).await?;

    // Find the appropriate asset for the target platform
    let asset_pattern = get_typst_asset_pattern(&config.target_os, &config.target_arch);
    let asset = release.assets.iter()
        .find(|a| a.name.contains(&asset_pattern))
        .ok_or_else(|| {
            let available_assets: Vec<String> = release.assets.iter()
                .map(|a| a.name.clone())
                .collect();
            format!(
                "No compatible Typst asset found for {}-{}.\nAvailable assets: {}\nLooked for pattern: {}",
                config.target_os, config.target_arch,
                available_assets.join(", "),
                asset_pattern
            )
        })?;

    let download_path = download_dir.join(&asset.name);

    // Try different mirrors if enabled (GitHub mirrors also work for other repos)
    if config.use_mirrors {
        for mirror in DOWNLOAD_MIRRORS {
            let download_url = construct_mirror_url(mirror, &asset.download_url);
            println!(
                "Trying to download {} from mirror: {}",
                asset.name,
                if mirror.is_empty() { "GitHub" } else { mirror }
            );

            match download_file(&download_url, &download_path).await {
                Ok(path) => return Ok(path),
                Err(e) => {
                    println!(
                        "Mirror {} failed: {}",
                        if mirror.is_empty() { "GitHub" } else { mirror },
                        e
                    );
                    continue;
                }
            }
        }
        Err("All download mirrors failed".to_string())
    } else {
        download_file(&asset.download_url, &download_path).await
    }
}

/// Get pandoc asset patterns for specific platform
fn get_pandoc_asset_patterns_for_platform(target_os: &str, target_arch: &str) -> Vec<String> {
    // Based on pandoc release assets like 'pandoc-3.7.0.2-x86_64-macOS.zip'
    match (target_os, target_arch) {
        ("windows", "x86_64") => vec!["windows-x86_64".to_string()],
        ("macos", "aarch64") => vec!["arm64-macOS".to_string()],
        ("macos", "x86_64") => vec!["x86_64-macOS".to_string()],
        ("linux", "aarch64") => vec!["linux-arm64".to_string()],
        ("linux", "x86_64") => vec!["linux-amd64".to_string()],
        // Fallbacks
        ("windows", _) => vec!["windows-x86_64".to_string()],
        ("macos", _) => vec!["macOS".to_string()],
        ("linux", _) => vec!["linux-amd64".to_string()],
        _ => vec!["linux-amd64".to_string()],
    }
}

/// Get typst asset pattern for specific platform
fn get_typst_asset_pattern(target_os: &str, target_arch: &str) -> String {
    // Based on typst release assets like 'typst-v0.13.1-x86_64-apple-darwin.tar.gz'
    match (target_os, target_arch) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
        ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
        ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
        ("linux", "x86_64") => "x86_64-unknown-linux-musl".to_string(),
        ("linux", "aarch64") => "aarch64-unknown-linux-musl".to_string(),
        // Fallbacks
        _ => "x86_64-unknown-linux-musl".to_string(),
    }
}

/// Extract archive with unified logic
pub async fn extract_archive_unified(
    archive_path: PathBuf,
    extract_dir: PathBuf,
) -> Result<String, String> {
    // Create extraction directory
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extract directory: {}", e))?;

    let extension = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "zip" => extract_zip(&archive_path, &extract_dir),
        "gz" => {
            // Handle .tar.gz and .tar.xz
            let file_name = archive_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if file_name.contains(".tar.") {
                extract_tar_gz(&archive_path, &extract_dir)
            } else {
                Err(format!("Unsupported .gz format: {}", file_name))
            }
        }
        "xz" => {
            let file_name = archive_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if file_name.contains(".tar.") {
                extract_tar_xz(&archive_path, &extract_dir)
            } else {
                Err(format!("Unsupported .xz format: {}", file_name))
            }
        }
        _ => Err(format!("Unsupported archive format: {}", extension)),
    }
}

/// Extract TAR.XZ archive (for Typst)
fn extract_tar_xz(archive_path: &PathBuf, extract_dir: &PathBuf) -> Result<String, String> {
    use std::io::BufReader;

    let file =
        std::fs::File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut reader = BufReader::new(file);

    // Decompress XZ first
    let mut decompressed = Vec::new();
    lzma_rs::xz_decompress(&mut reader, &mut decompressed)
        .map_err(|e| format!("Failed to decompress XZ: {}", e))?;

    // Then extract TAR
    let mut archive = tar::Archive::new(std::io::Cursor::new(decompressed));
    archive
        .unpack(extract_dir)
        .map_err(|e| format!("Failed to extract TAR.XZ archive: {}", e))?;

    Ok(extract_dir.to_string_lossy().to_string())
}

/// Download Typst for current platform
#[tauri::command]
pub async fn download_typst(
    version: Option<String>,
    download_dir: String,
) -> Result<String, String> {
    let config = DownloadConfig::current_platform();
    download_tool(
        DownloadType::Typst,
        version,
        PathBuf::from(download_dir),
        config,
    )
    .await
}

/// Get latest Typst release information (public command)
#[tauri::command]
pub async fn get_latest_typst_release_info() -> Result<GithubRelease, String> {
    let config = DownloadConfig::current_platform();
    get_latest_typst_release(&config).await
}

/// Update managed pandoc by downloading latest version  
#[tauri::command]
pub async fn update_managed_pandoc(app_handle: AppHandle) -> Result<String, String> {
    // Get latest release
    let latest_release = get_latest_pandoc_release().await?;
    let version = latest_release.tag_name.clone();

    // Get resource directory
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))?;

    let pandoc_dir = resource_dir.join("pandoc");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&pandoc_dir)
        .map_err(|e| format!("Failed to create pandoc directory: {}", e))?;

    // Download pandoc to resource directory
    let config = DownloadConfig::current_platform();
    let download_path = download_tool(
        DownloadType::Pandoc,
        Some(version.clone()),
        pandoc_dir.clone(),
        config,
    )
    .await?;

    // Extract the archive (this will overwrite existing files)
    extract_archive_unified(PathBuf::from(download_path), pandoc_dir).await?;

    Ok(format!(
        "Successfully updated managed pandoc to version {}",
        version
    ))
}

/// Update managed typst by downloading latest version
#[tauri::command]
pub async fn update_managed_typst(app_handle: AppHandle) -> Result<String, String> {
    // Get latest release
    let config = DownloadConfig::current_platform();
    let latest_release = get_latest_typst_release(&config).await?;
    let version = latest_release.tag_name.clone();

    // Get resource directory
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))?;

    let typst_dir = resource_dir.join("typst");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&typst_dir)
        .map_err(|e| format!("Failed to create typst directory: {}", e))?;

    // Download typst to resource directory
    let download_path = download_tool(
        DownloadType::Typst,
        Some(version.clone()),
        typst_dir.clone(),
        config,
    )
    .await?;

    // Extract the archive
    extract_archive_unified(PathBuf::from(download_path), typst_dir).await?;

    Ok(format!(
        "Successfully updated managed typst to version {}",
        version
    ))
}
