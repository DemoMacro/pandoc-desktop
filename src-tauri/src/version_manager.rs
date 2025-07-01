use crate::types::{GithubAsset, GithubRelease, VersionInfo};
use crate::utils::{format_file_size, get_pandoc_asset_patterns};
use std::io::Write;
use std::path::PathBuf;
use tauri_plugin_http::reqwest;

const UNGH_API_BASE: &str = "https://ungh.cc/repos";
const PANDOC_REPO: &str = "jgm/pandoc";

/// Mirror URLs for downloading (in order of preference)
const DOWNLOAD_MIRRORS: &[&str] = &[
    "https://hub.gitmirror.com/", // 7ed.net mirror
    "https://gh.ddlc.top/",       // GitHub proxy
    "",                           // Original GitHub (empty prefix)
];

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
    let releases = get_pandoc_releases(Some(50)).await?;

    let release = releases
        .into_iter()
        .find(|r| r.tag_name == version)
        .ok_or_else(|| format!("Version {} not found", version))?;

    // Try to find the best matching asset using priority order
    let asset_patterns = get_pandoc_asset_patterns();
    let mut selected_asset = None;
    
    for pattern in &asset_patterns {
        if let Some(asset) = release.assets.iter().find(|a| a.name.contains(pattern)) {
            selected_asset = Some(asset);
            break;
        }
    }
    
    let asset = selected_asset.ok_or_else(|| {
        let available_assets: Vec<String> = release.assets.iter()
            .map(|a| a.name.clone())
            .collect();
        format!(
            "No compatible asset found for current platform.\nAvailable assets: {}\nLooked for patterns: {:?}",
            available_assets.join(", "),
            asset_patterns
        )
    })?;

    let download_path = PathBuf::from(&download_dir).join(&asset.name);

    // Try different mirrors
    for mirror in DOWNLOAD_MIRRORS {
        let download_url = construct_mirror_url(mirror, &asset.download_url);

        println!("Trying to download {} from mirror: {}", asset.name, mirror);

        match download_file(&download_url, &download_path).await {
            Ok(path) => {
                return Ok(path);
            }
            Err(e) => {
                println!("Mirror {} failed: {}", mirror, e);
                continue;
            }
        }
    }

    Err("All download mirrors failed".to_string())
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
    let archive_path = PathBuf::from(archive_path);
    let extract_dir = PathBuf::from(extract_dir);

    // Create extraction directory
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extract directory: {}", e))?;

    let extension = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "zip" => extract_zip(&archive_path, &extract_dir),
        "gz" => extract_tar_gz(&archive_path, &extract_dir),
        _ => Err(format!("Unsupported archive format: {}", extension)),
    }
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
