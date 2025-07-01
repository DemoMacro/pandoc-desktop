use std::env;
use std::fs;
use std::path::PathBuf;
use std::io::Read;

fn main() {
    // Download and prepare Pandoc as a resource (not embedded in exe)
    if let Err(e) = prepare_pandoc_resource() {
        println!("cargo:warning=Failed to prepare Pandoc resource: {}", e);
        // Continue build even if Pandoc preparation fails
    }

    tauri_build::build()
}

fn prepare_pandoc_resource() -> Result<(), Box<dyn std::error::Error>> {
    // Get target information from environment variables
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());
    
    println!("cargo:warning=Building for target: {} (OS: {}, Arch: {})", target, target_os, target_arch);
    
    // Create resources directory in src-tauri
    let resources_dir = PathBuf::from("resources");
    fs::create_dir_all(&resources_dir)?;
    
    // Check if Pandoc is already extracted
    let pandoc_dir = resources_dir.join("pandoc");
    let pandoc_exe = if target_os == "windows" { "pandoc.exe" } else { "pandoc" };
    
    // Look for pandoc executable in extracted directory
    let mut pandoc_found = false;
    if pandoc_dir.exists() {
        // Check if pandoc executable exists in any subdirectory
        if let Ok(entries) = fs::read_dir(&pandoc_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let exe_path = entry.path().join(&pandoc_exe);
                    if exe_path.exists() {
                        pandoc_found = true;
                        break;
                    }
                } else if entry.file_name() == pandoc_exe {
                    pandoc_found = true;
                    break;
                }
            }
        }
    }
    
    if pandoc_found {
        println!("cargo:warning=Pandoc already extracted in: {}", pandoc_dir.display());
        return Ok(());
    }
    
    println!("cargo:warning=Downloading Pandoc as resource for {}-{}...", target_os, target_arch);
    
    // Get the latest Pandoc release URL for this platform
    let download_url = get_pandoc_download_url(&target_os, &target_arch)?;
    
    // Download Pandoc archive directly to memory and extract
    let response = ureq::get(&download_url).call()?;
    let mut data = Vec::new();
    response.into_reader().read_to_end(&mut data)?;
    
    println!("cargo:warning=Downloaded Pandoc data ({} bytes), extracting...", data.len());
    
    // Extract directly to resources/pandoc directory
    let extract_path = resources_dir.join("pandoc");
    std::fs::create_dir_all(&extract_path)?;
    
    // Determine format from download URL
    if download_url.contains(".zip") {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&data))?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = extract_path.join(file.mangled_name());
            
            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    std::fs::create_dir_all(p)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
                
                // Set executable permission on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = outfile.metadata()?;
                    let mut permissions = metadata.permissions();
                    permissions.set_mode(permissions.mode() | 0o755);
                    std::fs::set_permissions(&outpath, permissions)?;
                }
            }
        }
    } else if download_url.contains(".tar.gz") {
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(std::io::Cursor::new(&data)));
        archive.unpack(&extract_path)?;
    } else {
        return Err(format!("Unsupported archive format in URL: {}", download_url).into());
    }
    
    println!("cargo:warning=Successfully extracted pandoc to {}", extract_path.display());
    
    // Tell Tauri to include this as a resource
    println!("cargo:rerun-if-changed=resources");
    
    Ok(())
}

fn get_pandoc_download_url(target_os: &str, target_arch: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Use UNGH API for better reliability
    let api_url = "https://ungh.cc/repos/jgm/pandoc/releases/latest";
    let response = ureq::get(api_url).call()?;
    let json: serde_json::Value = response.into_json()?;
    
    let assets = json["release"]["assets"].as_array()
        .ok_or("No assets found in release")?;
    
    // Find the appropriate asset pattern based on target platform
    let pattern = match (target_os, target_arch) {
        ("windows", "x86_64") => "windows-x86_64.zip",
        ("windows", _) => "windows-x86_64.zip", // Windows默认用x86_64
        ("macos", "aarch64") => "arm64-macOS.zip",
        ("macos", "x86_64") => "x86_64-macOS.zip",
        ("macos", _) => "x86_64-macOS.zip", // macOS默认用x86_64
        ("linux", "aarch64") => "linux-arm64.tar.gz",
        ("linux", "x86_64") => "linux-amd64.tar.gz",
        ("linux", _) => "linux-amd64.tar.gz", // Linux默认用amd64
        _ => {
            return Err(format!("Unsupported platform: {}-{}", target_os, target_arch).into());
        }
    };
    
    for asset in assets {
        let download_url = asset["downloadUrl"].as_str().unwrap_or("");
        
        // Extract filename from URL
        let filename = download_url.split('/').last().unwrap_or("");
        
        if filename.contains(pattern) {
            println!("cargo:warning=Found matching pandoc asset: {}", filename);
            return Ok(download_url.to_string());
        }
    }
    
    Err(format!("No matching asset found for pattern: {}", pattern).into())
}
