use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ui::ColorPalette;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ollama_host: String,
    pub chat_model: String,
    pub embedding_model: String,
    pub data_path: PathBuf,
    pub theme: Theme,
    pub privacy: PrivacySettings,
    pub debug_logging: bool,
    /// Optional path to Firefox profile directory for authenticated scraping.
    /// If None, auto-detection is attempted. Set to use existing login sessions.
    #[serde(default)]
    pub firefox_profile_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    MidnightCommander,
    Default,
    Dark,
    Minimal,
    Monokai,
    SolarizedDark,
}

impl Theme {
    pub fn all() -> Vec<Theme> {
        vec![
            Theme::MidnightCommander,
            Theme::Default,
            Theme::Dark,
            Theme::Minimal,
            Theme::Monokai,
            Theme::SolarizedDark,
        ]
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            Theme::MidnightCommander => "Midnight Commander",
            Theme::Default => "Default",
            Theme::Dark => "Dark",
            Theme::Minimal => "Minimal",
            Theme::Monokai => "Monokai",
            Theme::SolarizedDark => "Solarized Dark",
        }
    }
    
    pub fn next(&self) -> Theme {
        match self {
            Theme::MidnightCommander => Theme::Default,
            Theme::Default => Theme::Dark,
            Theme::Dark => Theme::Minimal,
            Theme::Minimal => Theme::Monokai,
            Theme::Monokai => Theme::SolarizedDark,
            Theme::SolarizedDark => Theme::MidnightCommander,
        }
    }
    
    pub fn to_palette(&self) -> ColorPalette {
        match self {
            Theme::MidnightCommander => ColorPalette::midnight_commander(),
            Theme::Default => ColorPalette::default(),
            Theme::Dark => ColorPalette::dark(),
            Theme::Minimal => ColorPalette::minimal(),
            Theme::Monokai => ColorPalette::monokai(),
            Theme::SolarizedDark => ColorPalette::solarized_dark(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub store_evidence: bool,
    pub store_draft_text: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost:11434".to_string(),
            chat_model: "qwen2.5:7b".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            data_path: PathBuf::from("./data"),
            theme: Theme::MidnightCommander,
            privacy: PrivacySettings {
                store_evidence: true,
                store_draft_text: true,
            },
            debug_logging: false,
            firefox_profile_path: None,
        }
    }
}

impl Config {
    pub fn load_or_default() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Ensure data directory exists
        std::fs::create_dir_all(&self.data_path)?;
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        // Store config locally in current directory
        Ok(PathBuf::from("./config.toml"))
    }
}
