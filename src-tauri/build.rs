use std::env;
use std::fs;
use std::path::PathBuf;

// Use the unified download logic from manager.rs
// Since build.rs can't directly use the manager module, we'll implement a simplified version
// that uses the same logic patterns but with build-time compatible dependencies

fn main() {
    // Download and prepare Pandoc as a resource
    if let Err(e) = prepare_tool_resource("pandoc") {
        println!("cargo:warning=Failed to prepare Pandoc resource: {}", e);
        // Continue build even if Pandoc preparation fails
    }

    // Download and prepare Typst as a resource
    if let Err(e) = prepare_tool_resource("typst") {
        println!("cargo:warning=Failed to prepare Typst resource: {}", e);
        // Continue build even if Typst preparation fails
    }

    tauri_build::build()
}

fn prepare_tool_resource(tool: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Get target information from environment variables
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());

    println!(
        "cargo:warning=Preparing {} for {}-{}",
        tool, target_os, target_arch
    );

    // Create resources directory in src-tauri
    let resources_dir = PathBuf::from("resources");
    fs::create_dir_all(&resources_dir)?;

    let tool_dir = resources_dir.join(tool);
    let exe_name = if target_os == "windows" {
        format!("{}.exe", tool)
    } else {
        tool.to_string()
    };

    // Check if tool is already extracted
    if is_tool_available(&tool_dir, &exe_name) {
        println!(
            "cargo:warning={} already available in: {}",
            tool,
            tool_dir.display()
        );
        return Ok(());
    }

    println!(
        "cargo:warning=Downloading {} for {}-{}...",
        tool, target_os, target_arch
    );

    // Get the latest release URL for this platform
    let download_url = get_tool_download_url(tool, &target_os, &target_arch)?;

    // Download tool archive
    let response = ureq::get(&download_url).call()?;
    let mut data = Vec::new();
    std::io::copy(&mut response.into_reader(), &mut data)?;

    println!(
        "cargo:warning=Downloaded {} ({} bytes), extracting...",
        tool,
        data.len()
    );

    // Extract directly to resources/tool directory
    fs::create_dir_all(&tool_dir)?;
    extract_archive_to_dir(&data, &download_url, &tool_dir)?;

    println!(
        "cargo:warning=Successfully extracted {} to {}",
        tool,
        tool_dir.display()
    );

    // Tell Tauri to include this as a resource
    println!("cargo:rerun-if-changed=resources");

    Ok(())
}

fn is_tool_available(tool_dir: &PathBuf, exe_name: &str) -> bool {
    if !tool_dir.exists() {
        return false;
    }

    // Check for executable in common locations
    let candidates = vec![
        tool_dir.join(exe_name),             // Direct path
        tool_dir.join("bin").join(exe_name), // In bin subdirectory
    ];

    for candidate in candidates {
        if candidate.exists() {
            return true;
        }
    }

    // Check in any subdirectory (version directories are common)
    if let Ok(entries) = fs::read_dir(tool_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let sub_candidates = vec![
                    entry.path().join(exe_name),
                    entry.path().join("bin").join(exe_name),
                ];

                for candidate in sub_candidates {
                    if candidate.exists() {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn get_tool_download_url(
    tool: &str,
    target_os: &str,
    target_arch: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match tool {
        "pandoc" => get_pandoc_download_url(target_os, target_arch),
        "typst" => get_typst_download_url(target_os, target_arch),
        _ => Err(format!("Unsupported tool: {}", tool).into()),
    }
}

fn get_pandoc_download_url(
    target_os: &str,
    target_arch: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Use UNGH API for better reliability and mirroring
    let api_url = "https://ungh.cc/repos/jgm/pandoc/releases/latest";
    let response = ureq::get(api_url).call()?;
    let json: serde_json::Value = response.into_json()?;

    let assets = json["release"]["assets"]
        .as_array()
        .ok_or("No assets found in pandoc release")?;

    // Updated asset patterns based on pandoc 3.7.0.2 release
    let patterns = get_pandoc_asset_patterns(target_os, target_arch);

    for pattern in &patterns {
        for asset in assets {
            let download_url = asset["downloadUrl"].as_str().unwrap_or("");
            let filename = download_url.split('/').last().unwrap_or("");

            if filename.contains(pattern) {
                println!("cargo:warning=Found matching pandoc asset: {}", filename);
                return Ok(download_url.to_string());
            }
        }
    }

    Err(format!(
        "No matching pandoc asset found for {}-{}",
        target_os, target_arch
    )
    .into())
}

fn get_typst_download_url(
    target_os: &str,
    target_arch: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Use GitHub API directly for Typst
    let api_url = "https://api.github.com/repos/typst/typst/releases/latest";
    let response = ureq::get(api_url)
        .set("User-Agent", "pandoc-desktop")
        .call()?;
    let json: serde_json::Value = response.into_json()?;

    let assets = json["assets"]
        .as_array()
        .ok_or("No assets found in typst release")?;

    // Get the correct pattern for Typst assets
    let pattern = get_typst_asset_pattern(target_os, target_arch);

    for asset in assets {
        let name = asset["name"].as_str().unwrap_or("");
        let download_url = asset["browser_download_url"].as_str().unwrap_or("");

        if name.contains(&pattern) {
            println!("cargo:warning=Found matching typst asset: {}", name);
            return Ok(download_url.to_string());
        }
    }

    Err(format!("No matching typst asset found for pattern: {}", pattern).into())
}

fn get_pandoc_asset_patterns(target_os: &str, target_arch: &str) -> Vec<String> {
    // Based on actual pandoc 3.7.0.2 release assets
    match (target_os, target_arch) {
        ("windows", "x86_64") => vec!["windows-x86_64.zip".to_string()],
        ("windows", _) => vec!["windows-x86_64.zip".to_string()],
        ("macos", "aarch64") => vec!["arm64-macOS.zip".to_string(), "macOS.zip".to_string()],
        ("macos", "x86_64") => vec!["x86_64-macOS.zip".to_string(), "macOS.zip".to_string()],
        ("macos", _) => vec!["macOS.zip".to_string()],
        ("linux", "aarch64") => vec![
            "linux-arm64.tar.gz".to_string(),
            "linux-amd64.tar.gz".to_string(),
        ],
        ("linux", "x86_64") => vec!["linux-amd64.tar.gz".to_string()],
        ("linux", _) => vec!["linux-amd64.tar.gz".to_string()],
        _ => vec!["linux-amd64.tar.gz".to_string()], // Default fallback
    }
}

fn get_typst_asset_pattern(target_os: &str, target_arch: &str) -> String {
    // Based on actual typst v0.13.1 release assets
    match (target_os, target_arch) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
        ("windows", _) => "x86_64-pc-windows-msvc".to_string(),
        ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
        ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
        ("macos", _) => "x86_64-apple-darwin".to_string(),
        ("linux", "aarch64") => "aarch64-unknown-linux-musl".to_string(),
        ("linux", "x86_64") => "x86_64-unknown-linux-musl".to_string(),
        ("linux", _) => "x86_64-unknown-linux-musl".to_string(),
        _ => "x86_64-unknown-linux-musl".to_string(),
    }
}

fn extract_archive_to_dir(
    data: &[u8],
    url: &str,
    extract_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    if url.contains(".zip") {
        extract_zip_from_memory(data, extract_dir)?;
    } else if url.contains(".tar.gz") {
        extract_tar_gz_from_memory(data, extract_dir)?;
    } else if url.contains(".tar.xz") {
        extract_tar_xz_from_memory(data, extract_dir)?;
    } else {
        return Err(format!("Unsupported archive format in URL: {}", url).into());
    }

    Ok(())
}

fn extract_zip_from_memory(
    data: &[u8],
    extract_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = extract_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)?;
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;

            // Set executable permission on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    if mode & 0o111 != 0 {
                        let mut permissions = outfile.metadata()?.permissions();
                        permissions.set_mode(mode);
                        fs::set_permissions(&outpath, permissions)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn extract_tar_gz_from_memory(
    data: &[u8],
    extract_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let gz_decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(data));
    let mut archive = tar::Archive::new(gz_decoder);
    archive.unpack(extract_dir)?;
    Ok(())
}

fn extract_tar_xz_from_memory(
    data: &[u8],
    extract_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Decompress XZ first
    let mut decompressed = Vec::new();
    lzma_rs::xz_decompress(&mut std::io::Cursor::new(data), &mut decompressed)?;

    // Then extract TAR
    let mut archive = tar::Archive::new(std::io::Cursor::new(decompressed));
    archive.unpack(extract_dir)?;
    Ok(())
}
