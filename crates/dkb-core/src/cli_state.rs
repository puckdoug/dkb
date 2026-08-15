use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliState {
    #[serde(default)]
    pub last_list: Vec<Uuid>,
    #[serde(default)]
    pub current: Option<Uuid>,
}

fn state_file_path(data_dir: &Path) -> PathBuf {
    match data_dir.parent() {
        Some(p) => p.join("cli_state.json"),
        None => data_dir.join("cli_state.json"),
    }
}

impl CliState {
    pub fn load(data_dir: &Path) -> Self {
        let path = state_file_path(data_dir);
        match std::fs::read_to_string(&path) {
            Ok(s) => match serde_json::from_str(&s) {
                Ok(state) => state,
                Err(_) => {
                    eprintln!("warning: corrupted cli_state.json, resetting to defaults");
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, data_dir: &Path) -> std::io::Result<()> {
        let path = state_file_path(data_dir);
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }

    pub fn set_current(&mut self, id: Uuid) {
        self.current = Some(id);
    }

    pub fn clear_current(&mut self) {
        self.current = None;
    }

    pub fn set_last_list(&mut self, ids: Vec<Uuid>) {
        self.last_list = ids;
    }

    pub fn resolve_index(&self, index: usize) -> Option<Uuid> {
        self.last_list.get(index).copied()
    }
}
