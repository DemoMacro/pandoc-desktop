mod manager;
mod pandoc;
mod types;
mod utils;

use manager::{
    check_bundled_pandoc_update, create_and_validate_custom_manager, discover_pandoc_sources,
    download_pandoc, download_typst, extract_pandoc_archive, get_best_pandoc_manager,
    get_latest_pandoc_release, get_latest_typst_release_info, get_pandoc_releases,
    get_version_info, update_bundled_pandoc, update_managed_pandoc, update_managed_typst,
};
use pandoc::{
    check_bundled_typst, check_pandoc_version, check_portable_pandoc, convert_with_pandoc,
    get_available_pdf_engines, get_bundled_pandoc_path, get_bundled_typst_path, get_pandoc_info,
    get_pandoc_info_with_portable, get_pandoc_path, install_portable_pandoc, setup_bundled_pandoc,
    validate_pandoc_path,
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
            get_available_pdf_engines,
            // Portable Pandoc commands
            check_portable_pandoc,
            install_portable_pandoc,
            // Bundled Pandoc commands
            setup_bundled_pandoc,
            get_bundled_pandoc_path,
            // Bundled Typst commands
            check_bundled_typst,
            get_bundled_typst_path,
            // Version management commands
            get_latest_pandoc_release,
            get_pandoc_releases,
            get_version_info,
            download_pandoc,
            extract_pandoc_archive,
            discover_pandoc_sources,
            get_best_pandoc_manager,
            create_and_validate_custom_manager,
            update_bundled_pandoc,
            check_bundled_pandoc_update,
            // Typst commands
            download_typst,
            get_latest_typst_release_info,
            update_managed_pandoc,
            update_managed_typst
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
