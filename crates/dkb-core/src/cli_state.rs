use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliState {
    #[serde(default)]
    pub last_list: Vec<Uuid>,
    #[serde(default)]
    pub current: Option<Uuid>,
}

impl CliState {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("cli_state.json");
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, data_dir: &Path) -> std::io::Result<()> {
        let path = data_dir.join("cli_state.json");
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
