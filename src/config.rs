use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
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
            let default_dir = Self::default_data_dir();
            std::fs::create_dir_all(config_path.parent().unwrap_or(Path::new(".")))?;
            let content = format!("data_dir = \"{}\"\n", default_dir.display());
            std::fs::write(config_path, content)?;
            return Ok(Self { data_dir: default_dir });
        }

        let content = std::fs::read_to_string(config_path)?;
        let data_dir_str = content
            .lines()
            .find_map(|line| {
                let line = line.trim();
                line.strip_prefix("data_dir")
                    .map(|s| s.trim_start())
                    .and_then(|s| s.strip_prefix('='))
                    .map(|s| s.trim())
                    .and_then(|s| {
                        s.trim_matches('"')
                            .trim_matches('\'')
                            .to_string()
                            .into()
                    })
            })
            .unwrap_or_else(|| Self::default_data_dir().to_string_lossy().to_string());

        let data_dir = Self::expand_tilde(&data_dir_str);

        Ok(Self { data_dir })
    }

    fn expand_tilde(path: &str) -> PathBuf {
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                return PathBuf::from(home).join(rest);
            }
        }
        PathBuf::from(path)
    }
}
