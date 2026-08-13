use dkb::config::Config;
use tempfile::TempDir;

#[test]
fn test_default_data_dir() {
    let dir = Config::default_data_dir();
    assert!(dir.to_string_lossy().contains("dkb"));
}

#[test]
fn test_config_load_creates_default_when_missing() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let config = Config::load_from(&config_path).unwrap();
    assert!(config_path.exists());
    assert!(config.data_dir.to_string_lossy().contains("dkb"));
}

#[test]
fn test_config_load_reads_existing() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    let custom_data_dir = tmp.path().join("custom_data");
    std::fs::write(
        &config_path,
        format!("data_dir = \"{}\"", custom_data_dir.display()),
    )
    .unwrap();

    let config = Config::load_from(&config_path).unwrap();
    assert_eq!(config.data_dir, custom_data_dir);
}

#[test]
fn test_config_expands_tilde() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    std::fs::write(&config_path, "data_dir = \"~/dkb-data\"\n").unwrap();

    let config = Config::load_from(&config_path).unwrap();
    assert!(!config.data_dir.to_string_lossy().contains("~"));
    assert!(config.data_dir.to_string_lossy().contains("dkb-data"));
}
