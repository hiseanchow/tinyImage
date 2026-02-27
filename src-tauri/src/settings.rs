use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Auto,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub api_key: String,
    pub notify_mode: NotifyMode,
    pub output_mode: OutputMode,
    pub output_directory: String,
    pub context_menu_enabled: bool,
    #[serde(default = "default_theme")]
    pub theme: Theme,
}

fn default_theme() -> Theme {
    Theme::Auto
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotifyMode {
    Dialog,
    Notification,
    Silent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Alongside,
    Overwrite,
    Directory,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            notify_mode: NotifyMode::Notification,
            output_mode: OutputMode::Alongside,
            output_directory: String::new(),
            // macOS：NSServices 通过 Info.plist 始终注册，默认与实际状态保持一致
            // Windows：需要手动写注册表才能启用，默认关闭
            context_menu_enabled: cfg!(target_os = "macos"),
            theme: Theme::Auto,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TinyImage")
        .join("settings.json")
}

pub fn load() -> AppSettings {
    let path = config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AppSettings::default()
    }
}

pub fn save(settings: &AppSettings) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(settings)?;
    fs::write(path, data)?;
    Ok(())
}
