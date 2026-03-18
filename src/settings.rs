use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub model_path: String,
    pub language: String,
    pub hotkey: HotkeyConfig,
    pub corrections: Vec<Correction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Correction {
    pub from: String,
    pub to: String,
}

impl HotkeyConfig {
    pub fn display(&self) -> String {
        let mods: Vec<&str> = self
            .modifiers
            .iter()
            .map(|m| match m.as_str() {
                "Alt" => "⌥",
                "Ctrl" => "⌃",
                "Shift" => "⇧",
                "Meta" => "⌘",
                other => other,
            })
            .collect();
        format!("{} {}", mods.join(""), self.key)
    }
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: vec!["Alt".to_string()],
            key: "Space".to_string(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model_path: default_model_path(),
            language: "en".to_string(),
            hotkey: HotkeyConfig::default(),
            corrections: vec![
                Correction {
                    from: "clod".into(),
                    to: "Claude".into(),
                },
                Correction {
                    from: "key speak".into(),
                    to: "KeySpeak".into(),
                },
            ],
        }
    }
}

fn default_model_path() -> String {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("keyspeak")
        .join("ggml-small.bin")
        .to_string_lossy()
        .to_string()
}

impl Settings {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("keyspeak")
            .join("settings.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let p = Self::config_path();
        std::fs::create_dir_all(p.parent().unwrap())?;
        std::fs::write(p, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
