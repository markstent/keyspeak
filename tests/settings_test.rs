use keyspeak::settings::{HotkeyConfig, Settings};

#[test]
fn default_hotkey_display_is_alt_space() {
    let h = HotkeyConfig::default();
    assert_eq!(h.display(), "⌥ Space");
}

#[test]
fn settings_serialise_and_deserialise() {
    let original = Settings::default();
    let json = serde_json::to_string(&original).expect("serialise failed");
    let restored: Settings = serde_json::from_str(&json).expect("deserialise failed");

    assert_eq!(original.language, restored.language);
    assert_eq!(original.hotkey.key, restored.hotkey.key);
    assert_eq!(original.hotkey.modifiers, restored.hotkey.modifiers);
    assert_eq!(original.corrections.len(), restored.corrections.len());
}

#[test]
fn default_corrections_include_claude() {
    let s = Settings::default();
    assert!(s
        .corrections
        .iter()
        .any(|c| c.from == "clod" && c.to == "Claude"));
}

#[test]
fn custom_hotkey_display() {
    let h = HotkeyConfig {
        modifiers: vec!["Ctrl".to_string(), "Shift".to_string()],
        key: "KeyR".to_string(),
    };
    assert_eq!(h.display(), "⌃⇧ KeyR");
}
