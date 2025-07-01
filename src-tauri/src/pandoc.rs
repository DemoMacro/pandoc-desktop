use crate::types::PandocInfo;
use crate::utils::{get_search_paths, validate_pandoc_executable};
use std::path::{Path, PathBuf};
use tauri::Manager;

/// Get the portable Pandoc directory path
fn get_portable_pandoc_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    Ok(app_data_dir.join("pandoc-portable"))
}

/// Get the expected path to portable Pandoc executable
fn get_portable_pandoc_path(app_handle: &tauri::AppHandle) -> Result<String, String> {
    let portable_dir = get_portable_pandoc_dir(app_handle)?;
    let exe_name = if cfg!(target_os = "windows") {
        "pandoc.exe"
    } else {
        "pandoc"
    };
    
    // Look for pandoc in bin subdirectory (common in extracted archives)
    let bin_path = portable_dir.join("bin").join(exe_name);
    if bin_path.exists() {
        return Ok(bin_path.to_string_lossy().to_string());
    }
    
    // Look for pandoc in root of portable directory
    let root_path = portable_dir.join(exe_name);
    if root_path.exists() {
        return Ok(root_path.to_string_lossy().to_string());
    }
    
    // Look for pandoc in any subdirectory (pandoc archives often have version dirs)
    if let Ok(entries) = std::fs::read_dir(&portable_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let sub_bin = entry.path().join("bin").join(exe_name);
                if sub_bin.exists() {
                    return Ok(sub_bin.to_string_lossy().to_string());
                }
                
                let sub_root = entry.path().join(exe_name);
                if sub_root.exists() {
                    return Ok(sub_root.to_string_lossy().to_string());
                }
            }
        }
    }
    
    Err("Portable Pandoc executable not found".to_string())
}

/// Check if portable Pandoc is available and working
#[tauri::command]
pub async fn check_portable_pandoc(app_handle: tauri::AppHandle) -> Result<bool, String> {
    match get_portable_pandoc_path(&app_handle) {
        Ok(pandoc_path) => Ok(validate_pandoc_executable(&pandoc_path)),
        Err(_) => Ok(false),
    }
}

/// Install portable pandoc with improved download selection
#[tauri::command]
pub async fn install_portable_pandoc(app_handle: tauri::AppHandle) -> Result<String, String> {
    // Get latest release
    let latest_release = crate::manager::get_latest_pandoc_release().await?;
    let version = latest_release.tag_name;
    
    // Get portable pandoc directory
    let portable_dir = get_portable_pandoc_dir(&app_handle)?;
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&portable_dir)
        .map_err(|e| format!("Failed to create portable directory: {}", e))?;
    
    // Download pandoc to portable directory
    let download_path = crate::manager::download_pandoc(
        version.clone(), 
        portable_dir.to_string_lossy().to_string()
    ).await?;
    
    // Extract the archive
    let extract_dir = portable_dir.to_string_lossy().to_string();
    let extracted_path = crate::manager::extract_pandoc_archive(
        download_path, 
        extract_dir
    ).await?;
    
    Ok(format!("Successfully installed portable Pandoc {} to {}", version, extracted_path))
}

/// Find all possible pandoc installations
pub fn find_all_pandoc_paths() -> Vec<String> {
    let mut valid_paths = Vec::new();
    let search_paths = get_search_paths();

    for path in search_paths {
        if validate_pandoc_executable(&path) {
            // Check for duplicates before adding
            if !valid_paths.contains(&path) {
                valid_paths.push(path);
            }
        }
    }

    valid_paths
}

/// Enhanced get pandoc info that checks bundled and portable installation first
#[tauri::command]
pub async fn get_pandoc_info_with_portable(
    app_handle: tauri::AppHandle,
    custom_path: Option<String>,
) -> Result<PandocInfo, String> {
    // First try bundled pandoc (highest priority)
    if let Ok(bundled_path) = get_bundled_pandoc_executable_path(&app_handle) {
        if validate_pandoc_executable(&bundled_path) {
            println!("Using bundled Pandoc: {}", bundled_path);
            return get_pandoc_info(Some(bundled_path)).await;
        }
    }
    
    // Then try portable pandoc
    if let Ok(portable_path) = get_portable_pandoc_path(&app_handle) {
        if validate_pandoc_executable(&portable_path) {
            println!("Using portable Pandoc: {}", portable_path);
            return get_pandoc_info(Some(portable_path)).await;
        }
    }
    
    // Fallback to regular detection
    get_pandoc_info(custom_path).await
}

/// Get comprehensive pandoc information
#[tauri::command]
pub async fn get_pandoc_info(custom_path: Option<String>) -> Result<PandocInfo, String> {
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
            search_paths
                .iter()
                .take(5)
                .map(|p| format!("- {}", p))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    };

    // Check if pandoc exists and get version
    let version_output = crate::utils::create_hidden_command(&pandoc_cmd).arg("--version").output();

    match version_output {
        Ok(output) if output.status.success() => {
            let version_text = String::from_utf8(output.stdout)
                .map_err(|_| "Failed to read pandoc version output".to_string())?;

            let raw_version = version_text
                .lines()
                .next()
                .unwrap_or("Unknown version")
                .to_string();

            // Extract clean version number
            let version = extract_version_number(&raw_version);

            // Get supported formats
            let (input_formats, output_formats) = get_supported_formats(&pandoc_cmd)?;

            Ok(PandocInfo {
                version,
                path: pandoc_cmd,
                is_working: true,
                supported_input_formats: input_formats,
                supported_output_formats: output_formats,
                detected_paths,
                search_paths,
            })
        }
        Ok(_) => Err(format!("Pandoc at '{}' failed to execute", pandoc_cmd)),
        Err(e) => Err(format!(
            "Failed to execute pandoc at '{}': {}",
            pandoc_cmd, e
        )),
    }
}

/// Enhanced pandoc path detection with common installation paths
#[tauri::command]
pub async fn get_pandoc_path() -> Result<String, String> {
    let detected_paths = find_all_pandoc_paths();

    if detected_paths.is_empty() {
        Err("Pandoc not found in any common locations. Searched paths will be shown in detailed info.".to_string())
    } else {
        Ok(detected_paths[0].clone())
    }
}

/// Check if a custom path is valid
#[tauri::command]
pub async fn validate_pandoc_path(path: String) -> Result<bool, String> {
    if !Path::new(&path).exists() {
        return Ok(false);
    }

    let output = crate::utils::create_hidden_command(&path).arg("--version").output();

    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

/// Get supported input and output formats with improved error handling
pub fn get_supported_formats(pandoc_cmd: &str) -> Result<(Vec<String>, Vec<String>), String> {
    // Real Pandoc 3.7.0.2 formats as fallback (based on actual output)
    let fallback_input_formats = vec![
        "biblatex".to_string(),
        "bibtex".to_string(),
        "bits".to_string(),
        "commonmark".to_string(),
        "commonmark_x".to_string(),
        "creole".to_string(),
        "csljson".to_string(),
        "csv".to_string(),
        "djot".to_string(),
        "docbook".to_string(),
        "docx".to_string(),
        "dokuwiki".to_string(),
        "endnotexml".to_string(),
        "epub".to_string(),
        "fb2".to_string(),
        "gfm".to_string(),
        "haddock".to_string(),
        "html".to_string(),
        "ipynb".to_string(),
        "jats".to_string(),
        "jira".to_string(),
        "json".to_string(),
        "latex".to_string(),
        "man".to_string(),
        "markdown".to_string(),
        "markdown_github".to_string(),
        "markdown_mmd".to_string(),
        "markdown_phpextra".to_string(),
        "markdown_strict".to_string(),
        "mdoc".to_string(),
        "mediawiki".to_string(),
        "muse".to_string(),
        "native".to_string(),
        "odt".to_string(),
        "opml".to_string(),
        "org".to_string(),
        "pod".to_string(),
        "ris".to_string(),
        "rst".to_string(),
        "rtf".to_string(),
        "t2t".to_string(),
        "textile".to_string(),
        "tikiwiki".to_string(),
        "tsv".to_string(),
        "twiki".to_string(),
        "typst".to_string(),
        "vimwiki".to_string(),
    ];

    let fallback_output_formats = vec![
        "ansi".to_string(),
        "asciidoc".to_string(),
        "asciidoc_legacy".to_string(),
        "asciidoctor".to_string(),
        "beamer".to_string(),
        "biblatex".to_string(),
        "bibtex".to_string(),
        "chunkedhtml".to_string(),
        "commonmark".to_string(),
        "commonmark_x".to_string(),
        "context".to_string(),
        "csljson".to_string(),
        "djot".to_string(),
        "docbook".to_string(),
        "docbook4".to_string(),
        "docbook5".to_string(),
        "docx".to_string(),
        "dokuwiki".to_string(),
        "dzslides".to_string(),
        "epub".to_string(),
        "epub2".to_string(),
        "epub3".to_string(),
        "fb2".to_string(),
        "gfm".to_string(),
        "haddock".to_string(),
        "html".to_string(),
        "html4".to_string(),
        "html5".to_string(),
        "icml".to_string(),
        "ipynb".to_string(),
        "jats".to_string(),
        "jats_archiving".to_string(),
        "jats_articleauthoring".to_string(),
        "jats_publishing".to_string(),
        "jira".to_string(),
        "json".to_string(),
        "latex".to_string(),
        "man".to_string(),
        "markdown".to_string(),
        "markdown_github".to_string(),
        "markdown_mmd".to_string(),
        "markdown_phpextra".to_string(),
        "markdown_strict".to_string(),
        "markua".to_string(),
        "mediawiki".to_string(),
        "ms".to_string(),
        "muse".to_string(),
        "native".to_string(),
        "odt".to_string(),
        "opendocument".to_string(),
        "opml".to_string(),
        "org".to_string(),
        "pdf".to_string(),
        "plain".to_string(),
        "pptx".to_string(),
        "revealjs".to_string(),
        "rst".to_string(),
        "rtf".to_string(),
        "s5".to_string(),
        "slideous".to_string(),
        "slidy".to_string(),
        "tei".to_string(),
        "texinfo".to_string(),
        "textile".to_string(),
        "typst".to_string(),
        "xwiki".to_string(),
        "zimwiki".to_string(),
    ];

    // Try to get input formats with hidden window
    let input_formats = crate::utils::create_hidden_command(pandoc_cmd)
        .args(&["--list-input-formats"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|content| {
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

    // Try to get output formats with hidden window
    let output_formats = crate::utils::create_hidden_command(pandoc_cmd)
        .args(&["--list-output-formats"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|content| {
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

/// Helper function to find pandoc with same priority as detection
fn find_pandoc_with_priority(app_handle: &tauri::AppHandle) -> Result<String, String> {
    // 1. Try bundled pandoc first
    if let Ok(bundled_path) = get_bundled_pandoc_executable_path(app_handle) {
        if validate_pandoc_executable(&bundled_path) {
            return Ok(bundled_path);
        }
    }
    
    // 2. Try portable pandoc
    if let Ok(portable_path) = get_portable_pandoc_path(app_handle) {
        if validate_pandoc_executable(&portable_path) {
            return Ok(portable_path);
        }
    }
    
    // 3. Try system paths
    let detected_paths = find_all_pandoc_paths();
    if detected_paths.is_empty() {
        return Err("Pandoc not found. Please check your installation or specify a custom path in settings.".to_string());
    }
    Ok(detected_paths[0].clone())
}

/// Enhanced pandoc conversion with improved PDF engine handling
#[tauri::command]
pub async fn convert_with_pandoc(
    input_file: String,
    output_file: String,
    input_format: Option<String>,
    output_format: String,
    custom_pandoc_path: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Determine which pandoc path to use with same priority as detection
    let pandoc_cmd = if let Some(custom_path) = custom_pandoc_path {
        custom_path
    } else {
        find_pandoc_with_priority(&app_handle)?
    };

    // Validate output format only (input format is optional for auto-detection)
    let (_, supported_outputs) = get_supported_formats(&pandoc_cmd).unwrap_or_else(|_| {
        // If format detection fails, use fallback formats
        (
            vec![],
            vec![
                "html".to_string(),
                "latex".to_string(),
                "pdf".to_string(),
                "docx".to_string(),
                "epub".to_string(),
                "rst".to_string(),
                "markdown".to_string(),
                "plain".to_string(),
            ],
        )
    });

    if !supported_outputs.contains(&output_format) {
        return Err(format!(
            "Output format '{}' is not supported by this Pandoc installation",
            output_format
        ));
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

    // Special handling for PDF output
    if output_format == "pdf" {
        // Try alternative PDF engines if default fails
        args.push("--pdf-engine".to_string());
        
        // Check which PDF engines are available
        let available_engines = check_available_pdf_engines();
        if let Some(engine) = available_engines.first() {
            args.push(engine.clone());
        } else {
            return Err(format!(
                "PDF conversion failed: No PDF engine found.\n\n\
                To convert to PDF, please install one of the following:\n\
                • LaTeX distribution (TeX Live, MiKTeX): provides pdflatex, xelatex, lualatex\n\
                • wkhtmltopdf: lightweight HTML to PDF converter\n\
                • weasyprint: Python-based PDF generator\n\n\
                Alternatively, try converting to HTML first, then use a browser to print to PDF."
            ));
        }
    }

    // Add input and output files
    args.push(input_file.clone());
    args.push("-o".to_string());
    args.push(output_file.clone());

    // Execute conversion
    let output = crate::utils::create_hidden_command(&pandoc_cmd)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute pandoc at '{}': {}", pandoc_cmd, e))?;

    if output.status.success() {
        Ok(format!(
            "Successfully converted {} to {}",
            input_file, output_file
        ))
    } else {
        let error_msg =
            String::from_utf8(output.stderr).unwrap_or_else(|_| "Unknown pandoc error".to_string());
        
        // Provide helpful error messages for common PDF issues
        if output_format == "pdf" && error_msg.contains("pdflatex not found") {
            Err(format!(
                "PDF conversion failed: LaTeX not found.\n\n\
                Solutions:\n\
                1. Install a LaTeX distribution:\n\
                   • Windows: MiKTeX (https://miktex.org/)\n\
                   • macOS: MacTeX (https://www.tug.org/mactex/)\n\
                   • Linux: TeX Live (sudo apt install texlive-full)\n\n\
                2. Alternative: Convert to HTML first, then print to PDF from browser\n\n\
                3. Install wkhtmltopdf for direct HTML→PDF conversion\n\n\
                Original error: {}", error_msg.trim()
            ))
        } else {
            Err(format!("Pandoc conversion failed: {}", error_msg.trim()))
        }
    }
}

/// Check which PDF engines are available on the system
fn check_available_pdf_engines() -> Vec<String> {
    let engines = vec!["pdflatex", "xelatex", "lualatex", "wkhtmltopdf", "weasyprint"];
    let mut available = Vec::new();
    
    for engine in engines {
        if crate::utils::create_hidden_command(engine)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            available.push(engine.to_string());
        }
    }
    
    available
}

/// Extract clean version number from version string
fn extract_version_number(version_str: &str) -> String {
    // Handle common format: "pandoc.exe 3.7.0.2" or "pandoc 3.7.0.2"
    let parts: Vec<&str> = version_str.trim().split_whitespace().collect();

    // Look for version number in the parts
    for part in &parts {
        // Skip program names
        if part.contains("pandoc") {
            continue;
        }

        // Check if this part looks like a version number
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

    // Fallback: try to extract from the whole string
    let cleaned = version_str
        .trim()
        .replace("pandoc.exe", "")
        .replace("pandoc", "")
        .trim()
        .trim_start_matches('v')
        .trim()
        .to_string();

    if !cleaned.is_empty() {
        cleaned
    } else {
        "Unknown".to_string()
    }
}

/// Legacy function for compatibility - returns clean version number
#[tauri::command]
pub async fn check_pandoc_version() -> Result<String, String> {
    let info = get_pandoc_info(None).await?;
    Ok(extract_version_number(&info.version))
}

/// Get the bundled Pandoc directory path
fn get_bundled_pandoc_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    Ok(app_data_dir.join("pandoc-bundled"))
}

/// Setup bundled Pandoc from build-time resources
#[tauri::command]
pub async fn setup_bundled_pandoc(app_handle: tauri::AppHandle) -> Result<String, String> {
    let bundled_dir = get_bundled_pandoc_dir(&app_handle)?;
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&bundled_dir)
        .map_err(|e| format!("Failed to create bundled directory: {}", e))?;
    
    // Check if Pandoc is already set up
    let pandoc_path = get_bundled_pandoc_executable_path(&app_handle)?;
    if std::path::Path::new(&pandoc_path).exists() {
        return Ok(format!("Bundled Pandoc already available at: {}", pandoc_path));
    }
    
    // Check if we have a bundled Pandoc resource
    if let Ok(bundled_path) = std::env::var("PANDOC_BUNDLED_PATH") {
        if std::path::Path::new(&bundled_path).exists() {
            // Extract the bundled Pandoc
            let extract_dir = bundled_dir.to_string_lossy().to_string();
            let _extracted_path = crate::manager::extract_pandoc_archive(
                bundled_path.clone(),
                extract_dir
            ).await?;
            
            return Ok(format!("Successfully set up bundled Pandoc from: {}", bundled_path));
        }
    }
    
    Err("No bundled Pandoc found. This build may not include Pandoc.".to_string())
}

/// Get the path to bundled Pandoc executable
#[tauri::command]
pub async fn get_bundled_pandoc_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    get_bundled_pandoc_executable_path(&app_handle)
}

/// Get the expected path to bundled Pandoc executable
fn get_bundled_pandoc_executable_path(app_handle: &tauri::AppHandle) -> Result<String, String> {
    let bundled_dir = get_bundled_pandoc_dir(app_handle)?;
    let exe_name = if cfg!(target_os = "windows") {
        "pandoc.exe"
    } else {
        "pandoc"
    };
    
    // Look for pandoc in bin subdirectory (common in extracted archives)
    let bin_path = bundled_dir.join("bin").join(exe_name);
    if bin_path.exists() {
        return Ok(bin_path.to_string_lossy().to_string());
    }
    
    // Look for pandoc in root of bundled directory
    let root_path = bundled_dir.join(exe_name);
    if root_path.exists() {
        return Ok(root_path.to_string_lossy().to_string());
    }
    
    // Look for pandoc in any subdirectory (pandoc archives often have version dirs)
    if let Ok(entries) = std::fs::read_dir(&bundled_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let sub_bin = entry.path().join("bin").join(exe_name);
                if sub_bin.exists() {
                    return Ok(sub_bin.to_string_lossy().to_string());
                }
                
                let sub_root = entry.path().join(exe_name);
                if sub_root.exists() {
                    return Ok(sub_root.to_string_lossy().to_string());
                }
            }
        }
    }
    
    Err("Bundled Pandoc executable not found".to_string())
}
