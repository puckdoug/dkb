use std::path::Path;

pub fn init_workspace(data_dir: &Path) -> std::io::Result<()> {
    let iwe_dir = data_dir.join(".iwe");
    std::fs::create_dir_all(&iwe_dir)?;

    let config_path = iwe_dir.join("config.toml");
    if !config_path.exists() {
        let default_config = r#"[workspace]
name = "dkb"
"#;
        std::fs::write(config_path, default_config)?;
    }

    Ok(())
}
