use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Download Pandoc during build if needed
    if let Err(e) = download_pandoc_for_build() {
        println!("cargo:warning=Failed to download Pandoc during build: {}", e);
        // Continue build even if Pandoc download fails
    }

    tauri_build::build()
}

fn download_pandoc_for_build() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let resources_dir = PathBuf::from(&out_dir).join("resources");
    
    // Create resources directory if it doesn't exist
    fs::create_dir_all(&resources_dir)?;
    
    // Check if Pandoc is already downloaded
    let pandoc_filename = get_pandoc_filename();
    let pandoc_path = resources_dir.join(&pandoc_filename);
    
    if pandoc_path.exists() {
        println!("cargo:warning=Pandoc already exists at: {}", pandoc_path.display());
        return Ok(());
    }
    
    println!("cargo:warning=Downloading Pandoc for bundling...");
    
    // Get the latest Pandoc release URL
    let download_url = get_pandoc_download_url()?;
    
    // Download Pandoc
    let response = ureq::get(&download_url).call()?;
    let mut file = fs::File::create(&pandoc_path)?;
    std::io::copy(&mut response.into_reader(), &mut file)?;
    
    println!("cargo:warning=Downloaded Pandoc to: {}", pandoc_path.display());
    
    // Add as Tauri resource
    println!("cargo:rustc-env=PANDOC_BUNDLED_PATH={}", pandoc_path.display());
    
    Ok(())
}

fn get_pandoc_filename() -> String {
    if cfg!(target_os = "windows") {
        "pandoc-bundled.zip".to_string()
    } else if cfg!(target_os = "macos") {
        "pandoc-bundled.zip".to_string()
    } else {
        "pandoc-bundled.tar.gz".to_string()
    }
}

fn get_pandoc_download_url() -> Result<String, Box<dyn std::error::Error>> {
    // Use GitHub API to get the latest release
    let api_url = "https://api.github.com/repos/jgm/pandoc/releases/latest";
    let response = ureq::get(api_url).call()?;
    let json: serde_json::Value = response.into_json()?;
    
    let assets = json["assets"].as_array()
        .ok_or("No assets found in release")?;
    
    // Find the appropriate asset for current platform
    let pattern = if cfg!(target_os = "windows") {
        "windows-x86_64.zip"
    } else if cfg!(target_os = "macos") {
        "macOS.zip"
    } else {
        "linux-amd64.tar.gz"
    };
    
    for asset in assets {
        let name = asset["name"].as_str().unwrap_or("");
        if name.contains(pattern) {
            let download_url = asset["browser_download_url"].as_str()
                .ok_or("No download URL found")?;
            return Ok(download_url.to_string());
        }
    }
    
    Err("No matching asset found for current platform".into())
}
