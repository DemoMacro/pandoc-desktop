mod pandoc;
mod types;
mod utils;
mod version_manager;

use pandoc::{
    check_pandoc_version, convert_with_pandoc, get_pandoc_info, get_pandoc_path,
    validate_pandoc_path, check_portable_pandoc, install_portable_pandoc, 
    get_pandoc_info_with_portable, setup_bundled_pandoc, get_bundled_pandoc_path,
};
use version_manager::{
    download_pandoc, extract_pandoc_archive, get_latest_pandoc_release, get_pandoc_releases,
    get_version_info,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Pandoc detection and conversion commands
            get_pandoc_info,
            get_pandoc_info_with_portable,
            get_pandoc_path,
            validate_pandoc_path,
            convert_with_pandoc,
            check_pandoc_version,
            // Portable Pandoc commands
            check_portable_pandoc,
            install_portable_pandoc,
            // Bundled Pandoc commands
            setup_bundled_pandoc,
            get_bundled_pandoc_path,
            // Version management commands
            get_latest_pandoc_release,
            get_pandoc_releases,
            get_version_info,
            download_pandoc,
            extract_pandoc_archive
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
