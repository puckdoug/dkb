#![allow(clippy::pedantic)]

use dkb_core::viewer::{
    build_open_command, detect_default_viewer, resolve_viewer_path, ViewerPreference,
};
use std::path::PathBuf;

#[test]
fn test_viewer_preference_default() {
    assert_eq!(ViewerPreference::default(), ViewerPreference::Auto);
}

#[test]
fn test_viewer_preference_serialization() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Wrapper {
        viewer: ViewerPreference,
    }

    let auto = Wrapper {
        viewer: ViewerPreference::Auto,
    };
    let toml_str = toml::to_string(&auto).unwrap();
    let deserialized: Wrapper = toml::from_str(&toml_str).unwrap();
    assert_eq!(deserialized, auto);

    let custom = Wrapper {
        viewer: ViewerPreference::Custom(PathBuf::from("/Applications/Marked 2.app")),
    };
    let toml_str_custom = toml::to_string(&custom).unwrap();
    let deserialized_custom: Wrapper = toml::from_str(&toml_str_custom).unwrap();
    assert_eq!(deserialized_custom, custom);
}

#[test]
fn test_viewer_custom_path() {
    let custom = ViewerPreference::Custom(PathBuf::from("/Applications/Marked 2.app"));
    assert_eq!(
        resolve_viewer_path(&custom),
        Some(PathBuf::from("/Applications/Marked 2.app"))
    );
}

#[test]
fn test_viewer_priority_ordering() {
    let pref_auto = ViewerPreference::Auto;
    let resolved = resolve_viewer_path(&pref_auto);
    let detected = detect_default_viewer();
    assert_eq!(resolved, detected);

    if let Some(path) = resolved {
        assert!(path.to_string_lossy().ends_with(".app"));
        assert!(path.exists());
    }
}

#[test]
fn test_build_open_command_with_custom_viewer() {
    let file_path = PathBuf::from("/tmp/test.md");
    let pref = ViewerPreference::Custom(PathBuf::from("/Applications/Marked 2.app"));
    let cmd = build_open_command(&file_path, &pref);
    let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
    assert_eq!(args, vec!["-a", "/Applications/Marked 2.app", "/tmp/test.md"]);
}

#[test]
fn test_build_open_command_with_no_viewer() {
    let file_path = PathBuf::from("/tmp/test.md");
    // When custom path is not set and auto resolves to None (or mock None)
    // build_open_command_with_app should produce "open <file>"
    let cmd = dkb_core::viewer::build_open_command_with_app(&file_path, None);
    let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
    assert_eq!(args, vec!["/tmp/test.md"]);
}
