use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub volume: f32,
    pub last_url: Option<String>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub close_to_tray: bool,
    #[serde(default)]
    pub output_device: Option<String>,
    #[serde(default = "default_skip_ads")]
    pub skip_ads: bool,
    #[serde(default)]
    pub discord_rpc: bool,
    #[serde(default)]
    pub auto_identify: bool,
    #[serde(default = "default_auto_identify_cooldown_success")]
    pub auto_identify_cooldown_success: u32,
    #[serde(default = "default_auto_identify_cooldown_fail")]
    pub auto_identify_cooldown_fail: u32,
    /// Whether the hidden "main"/"tray" windows get destroyed after sitting idle, to free
    /// the memory their WebView holds. Disabling keeps them alive (instant reopen, more RAM).
    #[serde(default = "default_true")]
    pub main_idle_destroy_enabled: bool,
    #[serde(default = "default_main_idle_grace_secs")]
    pub main_idle_grace_secs: u32,
    #[serde(default = "default_true")]
    pub tray_idle_destroy_enabled: bool,
    #[serde(default = "default_tray_idle_grace_secs")]
    pub tray_idle_grace_secs: u32,
}

fn default_skip_ads() -> bool { true }
fn default_auto_identify_cooldown_success() -> u32 { 60 }
fn default_auto_identify_cooldown_fail() -> u32 { 30 }
fn default_true() -> bool { true }
fn default_main_idle_grace_secs() -> u32 { 300 }
fn default_tray_idle_grace_secs() -> u32 { 30 }


impl Default for Settings {
    fn default() -> Self {
        Self {
            volume: 0.8,
            last_url: None,
            sort_by: Some("manual".to_string()),
            sort_order: Some("asc".to_string()),
            language: None,
            theme: None,
            minimize_to_tray: true,
            close_to_tray: false,
            output_device: None,
            skip_ads: true,
            discord_rpc: false,
            auto_identify: false,
            auto_identify_cooldown_success: 60,
            auto_identify_cooldown_fail: 30,
            main_idle_destroy_enabled: true,
            main_idle_grace_secs: 300,
            tray_idle_destroy_enabled: true,
            tray_idle_grace_secs: 30,
        }
    }
}

impl Settings {
    /// Load settings from disk. Returns defaults if file is missing or corrupt.
    pub fn load(app_data_dir: &Path) -> Self {
        let path = app_data_dir.join("settings.json");
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Persist settings to disk. Creates the directory if needed.
    pub fn save(&self, app_data_dir: &Path) -> Result<(), AppError> {
        fs::create_dir_all(app_data_dir).map_err(|e| AppError::Settings(e.to_string()))?;
        let path = app_data_dir.join("settings.json");
        let content =
            serde_json::to_string_pretty(self).map_err(|e| AppError::Settings(e.to_string()))?;
        fs::write(&path, content).map_err(|e| AppError::Settings(e.to_string()))?;
        Ok(())
    }
}
