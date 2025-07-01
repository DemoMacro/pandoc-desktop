use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PandocInfo {
    pub version: String,
    pub path: String,
    pub is_working: bool,
    pub supported_input_formats: Vec<String>,
    pub supported_output_formats: Vec<String>,
    pub detected_paths: Vec<String>,
    pub search_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PandocConfig {
    pub custom_path: Option<String>,
    pub use_custom_path: bool,
    pub last_detected_version: String,
}

impl Default for PandocConfig {
    fn default() -> Self {
        Self {
            custom_path: None,
            use_custom_path: false,
            last_detected_version: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub assets: Vec<GithubAsset>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GithubAsset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub content_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percentage: f64,
    pub speed: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionInfo {
    pub current: Option<String>,
    pub latest: Option<String>,
    pub available_versions: Vec<String>,
    pub is_update_available: bool,
}
