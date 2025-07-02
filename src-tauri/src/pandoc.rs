use crate::types::PandocInfo;
use crate::utils::{get_search_paths, validate_pandoc_executable};
use std::path::Path;
use tauri::Manager;

/// Get the default PDF engine for a given output format (based on pandoc manual)
fn get_default_pdf_engine(output_format: &str) -> &'static str {
    match output_format {
        "latex" => "pdflatex",
        "context" => "context",
        "html" => "wkhtmltopdf",
        "ms" => "pdfroff",
        "typst" => "typst",
        _ => "pdflatex", // Default for most cases
    }
}

/// Get available PDF engines for a given output format (in priority order)
fn get_pdf_engines_for_format(output_format: &str) -> Vec<&'static str> {
    let engines = match output_format {
        "latex" => vec!["pdflatex", "xelatex", "lualatex", "tectonic", "latexmk"],
        "context" => vec!["context"],
        "html" => vec!["wkhtmltopdf", "weasyprint", "prince", "pagedjs-cli"],
        "ms" => vec!["pdfroff"],
        "typst" => vec!["typst"],
        _ => vec![
            "pdflatex",
            "xelatex",
            "lualatex",
            "wkhtmltopdf",
            "weasyprint",
            "typst",
        ],
    };

    engines
}

/// Check if portable Pandoc is available and working
#[tauri::command]
pub async fn check_portable_pandoc(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let managed_source = crate::manager::PandocManager::new(crate::manager::PandocSource::Managed);
    Ok(managed_source.get_executable_path(&app_handle).is_some())
}

/// Install portable pandoc with improved download selection
#[tauri::command]
pub async fn install_portable_pandoc(app_handle: tauri::AppHandle) -> Result<String, String> {
    // Get latest release
    let latest_release = crate::manager::get_latest_pandoc_release().await?;
    let version = latest_release.tag_name;

    // Get app data directory for portable installation
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let portable_dir = app_data_dir.join("pandoc-portable");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&portable_dir)
        .map_err(|e| format!("Failed to create portable directory: {}", e))?;

    // Download pandoc to portable directory
    let download_path = crate::manager::download_pandoc(
        version.clone(),
        portable_dir.to_string_lossy().to_string(),
    )
    .await?;

    // Extract the archive
    let extract_dir = portable_dir.to_string_lossy().to_string();
    let extracted_path = crate::manager::extract_pandoc_archive(download_path, extract_dir).await?;

    Ok(format!(
        "Successfully installed portable Pandoc {} to {}",
        version, extracted_path
    ))
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

/// Enhanced get pandoc info that checks managed installation first
#[tauri::command]
pub async fn get_pandoc_info_with_portable(
    app_handle: tauri::AppHandle,
    custom_path: Option<String>,
) -> Result<PandocInfo, String> {
    // First try managed pandoc (unified bundled/portable)
    let managed_source = crate::manager::PandocManager::new(crate::manager::PandocSource::Managed);
    if let Some(managed_path) = managed_source.get_executable_path(&app_handle) {
        if validate_pandoc_executable(&managed_path.to_string_lossy()) {
            println!("Using managed Pandoc: {}", managed_path.display());
            return get_pandoc_info(Some(managed_path.to_string_lossy().to_string())).await;
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
    let version_output = crate::utils::create_hidden_command(&pandoc_cmd)
        .arg("--version")
        .output();

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

    let output = crate::utils::create_hidden_command(&path)
        .arg("--version")
        .output();

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

/// Helper function to find pandoc with unified priority logic
fn find_pandoc_with_priority(app_handle: &tauri::AppHandle) -> Result<String, String> {
    // 1. Try managed pandoc first (unified bundled/portable)
    let managed_source = crate::manager::PandocManager::new(crate::manager::PandocSource::Managed);
    if let Some(managed_path) = managed_source.get_executable_path(app_handle) {
        if validate_pandoc_executable(&managed_path.to_string_lossy()) {
            let path_str = managed_path.to_string_lossy().to_string();
            return Ok(path_str);
        }
    }

    // 2. Try system paths
    let detected_paths = find_all_pandoc_paths();

    if detected_paths.is_empty() {
        return Err("Pandoc not found. Please check your installation or specify a custom path in settings.".to_string());
    }

    let path_str = detected_paths[0].clone();
    Ok(path_str)
}

/// Enhanced pandoc conversion with correct PDF engine handling
#[tauri::command]
pub async fn convert_with_pandoc(
    input_file: String,
    output_file: String,
    input_format: Option<String>,
    output_format: String,
    custom_pandoc_path: Option<String>,
    pdf_engine: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Determine which pandoc path to use with unified priority logic
    let pandoc_cmd = if let Some(custom_path) = custom_pandoc_path {
        custom_path
    } else {
        let detected_path = find_pandoc_with_priority(&app_handle)?;
        detected_path
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

    // Special handling for PDF output with correct engine selection
    if output_format == "pdf" {
        args.push("--pdf-engine".to_string());

        // Use user-specified engine or determine best default for output format
        let engine_to_use = if let Some(user_engine) = pdf_engine {
            // Validate that the user-specified engine is available
            let available_engines =
                check_available_pdf_engines_for_format(&output_format, &app_handle);

            if available_engines.contains(&user_engine) {
                user_engine
            } else {
                return Err(format!(
                    "Specified PDF engine '{}' is not available.\n\
                     Available engines for PDF output: {}",
                    user_engine,
                    available_engines.join(", ")
                ));
            }
        } else {
            // Auto-select best available engine for this output format
            let default_engine = get_default_pdf_engine(&output_format);
            let available_engines =
                check_available_pdf_engines_for_format(&output_format, &app_handle);

            // Try default engine first, then fallback to any available
            if available_engines.contains(&default_engine.to_string()) {
                default_engine.to_string()
            } else if let Some(engine) = available_engines.first() {
                engine.clone()
            } else {
                return Err(format!(
                    "PDF conversion failed: No PDF engine found for output format '{}'.\n\n\
                     Recommended PDF engines for {}:\n\
                     {}\n\n\
                     Installation guides:\n\
                     • typst: 'cargo install typst-cli' or download from GitHub releases\n\
                     • wkhtmltopdf: Download from https://wkhtmltopdf.org/\n\
                     • weasyprint: 'pip install weasyprint'\n\
                     • LaTeX distribution (TeX Live, MiKTeX): For academic publishing\n\n\
                     Alternatively, try converting to HTML first, then use a browser to print to PDF.",
                     output_format,
                     output_format,
                     get_pdf_engines_for_format(&output_format).join(", ")
                ));
            }
        };

        // Use the engine directly (it may already be a full path from get_best_typst_path)
        args.push(engine_to_use);
    }

    // Add input and output files
    args.push(input_file.clone());
    args.push("-o".to_string());
    args.push(output_file.clone());

    // Execute conversion
    // Set proper working directory for pandoc execution
    let working_dir = std::path::Path::new(&input_file)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    let output = crate::utils::create_hidden_command(&pandoc_cmd)
        .args(&args)
        .current_dir(&working_dir)
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
        if output_format == "pdf" && error_msg.contains("not found") {
            Err(format!(
                "PDF conversion failed: Required engine not found.\n\n\
                Solutions:\n\
                1. Install the recommended PDF engine for this format\n\
                2. Alternative: Convert to HTML first, then print to PDF from browser\n\
                3. Try a different output format (html, docx, etc.)\n\n\
                Original error: {}",
                error_msg.trim()
            ))
        } else {
            Err(format!("Pandoc conversion failed: {}", error_msg.trim()))
        }
    }
}

/// Get available PDF engines for specific output format
#[tauri::command]
pub async fn get_available_pdf_engines(
    app_handle: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    // Default to "pdf" format if not specified
    Ok(check_available_pdf_engines_for_format("pdf", &app_handle))
}

/// Check which PDF engines are available for a specific output format
fn check_available_pdf_engines_for_format(
    output_format: &str,
    app_handle: &tauri::AppHandle,
) -> Vec<String> {
    let mut available = Vec::new();
    let engines = get_pdf_engines_for_format(output_format);

    for engine in engines {
        // Check bundled engines first (currently only typst)
        if engine == "typst" {
            if let Some(bundled_typst) = get_best_typst_path(app_handle) {
                if !available.contains(&bundled_typst) {
                    available.push(bundled_typst);
                }
                continue;
            }
        }

        // Check system engines
        let result = crate::utils::create_hidden_command(engine)
            .arg("--version")
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    available.push(engine.to_string());
                }
            }
            Err(_) => {
                // Engine not available, continue to next
            }
        }
    }

    available
}

/// Get the best available typst path (bundled or system) - returns full path when possible
fn get_best_typst_path(app_handle: &tauri::AppHandle) -> Option<String> {
    // Check bundled typst in multiple possible locations
    let exe_name = if cfg!(windows) { "typst.exe" } else { "typst" };

    // Try different possible resource directories
    let mut possible_resource_dirs = Vec::new();

    // 1. Try the official resource directory (works in production)
    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        possible_resource_dirs.push(resource_dir);
    }

    // 2. Try relative to the current working directory (development)
    if let Ok(current_dir) = std::env::current_dir() {
        // If we're already in src-tauri directory, don't add it again
        let resources_path = if current_dir.file_name() == Some(std::ffi::OsStr::new("src-tauri")) {
            current_dir.join("resources")
        } else {
            current_dir.join("src-tauri").join("resources")
        };
        possible_resource_dirs.push(resources_path);
    }

    // 3. Try relative to the executable directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_parent) = exe_path.parent() {
            possible_resource_dirs.push(exe_parent.join("resources"));
        }
    }

    for resource_dir in possible_resource_dirs {
        let typst_dir = resource_dir.join("typst");

        // Try direct path first
        let direct_path = typst_dir.join(&exe_name);
        if direct_path.exists() {
            if validate_typst_executable(&direct_path.to_string_lossy()) {
                let path_str = direct_path.to_string_lossy().to_string();
                return Some(path_str);
            }
        }

        // Try to find typst.exe in subdirectories (for extracted archives)
        if typst_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&typst_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let nested_typst = entry.path().join(&exe_name);

                        if nested_typst.exists() {
                            if validate_typst_executable(&nested_typst.to_string_lossy()) {
                                let path_str = nested_typst.to_string_lossy().to_string();
                                return Some(path_str);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to system typst - use command name only

    let result = crate::utils::create_hidden_command("typst")
        .arg("--version")
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Some("typst".to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    }
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

/// Setup managed Pandoc from build-time resources (legacy function kept for compatibility)
#[tauri::command]
pub async fn setup_bundled_pandoc(app_handle: tauri::AppHandle) -> Result<String, String> {
    let managed_source = crate::manager::PandocManager::new(crate::manager::PandocSource::Managed);

    if let Some(path) = managed_source.get_executable_path(&app_handle) {
        Ok(format!("Managed Pandoc available at: {}", path.display()))
    } else {
        Err("No managed Pandoc found. This build may not include Pandoc.".to_string())
    }
}

/// Get the path to managed Pandoc executable (legacy function kept for compatibility)
#[tauri::command]
pub async fn get_bundled_pandoc_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let managed_source = crate::manager::PandocManager::new(crate::manager::PandocSource::Managed);

    if let Some(path) = managed_source.get_executable_path(&app_handle) {
        Ok(path.to_string_lossy().to_string())
    } else {
        Err("Managed Pandoc executable not found".to_string())
    }
}

/// Check if managed typst is available
#[tauri::command]
pub async fn check_bundled_typst(app_handle: tauri::AppHandle) -> Result<bool, String> {
    Ok(get_best_typst_path(&app_handle).is_some())
}

/// Get the path to best available typst executable
#[tauri::command]
pub async fn get_bundled_typst_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    get_best_typst_path(&app_handle).ok_or_else(|| "Typst executable not found".to_string())
}

/// Validate if a Typst executable is working
fn validate_typst_executable(path: &str) -> bool {
    crate::utils::create_hidden_command(path)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
