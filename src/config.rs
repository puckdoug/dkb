use crate::i18n::Language;
use crate::viewer::ViewerPreference;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    #[serde(default)]
    pub vi_mode: bool,
    #[serde(default)]
    pub line_numbers: bool,
    #[serde(default)]
    pub theme_mode: ThemeMode,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub markdown_viewer: ViewerPreference,
}

impl Config {
    pub fn default_data_dir() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join("Library/Application Support/dkb")
        } else {
            PathBuf::from("Library/Application Support/dkb")
        }
    }

    pub fn config_file_path() -> PathBuf {
        Self::default_data_dir().join("config.toml")
    }

    pub fn load() -> std::io::Result<Self> {
        Self::load_from(&Self::config_file_path())
    }

    pub fn load_from(config_path: &Path) -> std::io::Result<Self> {
        if !config_path.exists() {
            let default_config = Self {
                data_dir: Self::default_data_dir(),
                vi_mode: false,
                line_numbers: false,
                theme_mode: ThemeMode::System,
                language: Language::Auto,
                markdown_viewer: ViewerPreference::Auto,
            };
            default_config.save_to(config_path)?;
            return Ok(default_config);
        }

        let content = std::fs::read_to_string(config_path)?;
        let mut config: Config = toml::from_str(&content)
            .unwrap_or_else(|_| Config {
                data_dir: Self::default_data_dir(),
                vi_mode: false,
                line_numbers: false,
                theme_mode: ThemeMode::System,
                language: Language::Auto,
                markdown_viewer: ViewerPreference::Auto,
            });
        
        config.data_dir = Self::expand_tilde(&config.data_dir.to_string_lossy());
        if let ViewerPreference::Custom(ref path) = config.markdown_viewer {
            config.markdown_viewer = ViewerPreference::Custom(Self::expand_tilde(&path.to_string_lossy()));
        }
        Ok(config)
    }

    pub fn save_to(&self, config_path: &Path) -> std::io::Result<()> {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    fn expand_tilde(path: &str) -> PathBuf {
        if let Some(rest) = path.strip_prefix("~/")
            && let Some(home) = std::env::var_os("HOME") {
                return PathBuf::from(home).join(rest);
            }
        PathBuf::from(path)
    }
}
