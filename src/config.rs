use serde::Deserialize;
use std::{fs, io};

#[derive(Debug, Deserialize, Default)]
pub struct ColorsConfig {
    pub text: Option<String>,
    pub background: Option<String>,
    pub status_bar_text: Option<String>,
    pub status_bar_background: Option<String>,
    pub command_box_text: Option<String>,
    pub command_box_background: Option<String>,
    pub command_box_border: Option<String>,
    pub message_text: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ModeBarConfig {
    pub show_mode: Option<bool>,
    pub show_filename: Option<bool>,
    pub show_dirty_indicator: Option<bool>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub text_color: Option<String>,
    pub height: Option<u16>,
    pub width: Option<u16>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CommandBoxConfig {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub text_color: Option<String>,
    pub height: Option<u16>,
    pub width: Option<u16>,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorsConfig,
    #[serde(rename = "mode bar", default)]
    pub mode_bar: ModeBarConfig,
    #[serde(rename = "command box", default)]
    pub command_box: CommandBoxConfig,
}

impl Config {
    pub fn load(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse TOML: {}", e)))?;
        Ok(config)
    }
}

