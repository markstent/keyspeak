//! egui settings window — hotkey, language, word corrections.

use crate::settings::{Correction, Settings};
use eframe::egui;

const MODIFIERS: &[&str] = &["Alt (⌥)", "Ctrl (⌃)", "Shift (⇧)", "Meta (⌘)", "Ctrl+Shift"];

const KEYS: &[&str] = &[
    "Space", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "KeyA",
    "KeyB", "KeyC", "KeyD", "KeyE", "KeyF", "KeyG", "KeyH", "KeyI", "KeyJ", "KeyK", "KeyL", "KeyM",
    "KeyN", "KeyO", "KeyP", "KeyQ", "KeyR", "KeyS", "KeyT", "KeyU", "KeyV", "KeyW", "KeyX", "KeyY",
    "KeyZ",
];

const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("fr", "French"),
    ("de", "German"),
    ("es", "Spanish"),
    ("it", "Italian"),
    ("pt", "Portuguese"),
    ("zh", "Chinese"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("ru", "Russian"),
    ("nl", "Dutch"),
    ("pl", "Polish"),
    ("ar", "Arabic"),
    ("hi", "Hindi"),
];

pub struct SettingsWindow {
    pub settings: Settings,
    pub pending_save: bool,
    pub hotkey_changed: bool,
    new_from: String,
    new_to: String,
    modifier_idx: usize,
    key_idx: usize,
    lang_idx: usize,
    status: Option<String>,
}

impl SettingsWindow {
    pub fn new(settings: Settings) -> Self {
        let modifier_idx = MODIFIERS
            .iter()
            .position(|&m| m.starts_with(first_modifier(&settings.hotkey.modifiers)))
            .unwrap_or(0);
        let key_idx = KEYS
            .iter()
            .position(|&k| k == settings.hotkey.key)
            .unwrap_or(0);
        let lang_idx = LANGUAGES
            .iter()
            .position(|(code, _)| *code == settings.language)
            .unwrap_or(0);

        Self {
            settings,
            pending_save: false,
            hotkey_changed: false,
            new_from: String::new(),
            new_to: String::new(),
            modifier_idx,
            key_idx,
            lang_idx,
            status: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("KeySpeak Settings");
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(10.0);

            // ── Hotkey ─────────────────────────────────────────────────────
            ui.label(egui::RichText::new("🎹  Record Hotkey").strong());
            ui.add_space(2.0);

            let prev = (self.modifier_idx, self.key_idx);
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("modifier")
                    .selected_text(MODIFIERS[self.modifier_idx])
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        for (i, &m) in MODIFIERS.iter().enumerate() {
                            ui.selectable_value(&mut self.modifier_idx, i, m);
                        }
                    });
                ui.label("+");
                egui::ComboBox::from_id_source("key")
                    .selected_text(KEYS[self.key_idx])
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        for (i, &k) in KEYS.iter().enumerate() {
                            ui.selectable_value(&mut self.key_idx, i, k);
                        }
                    });
            });

            if (self.modifier_idx, self.key_idx) != prev {
                self.hotkey_changed = true;
                self.sync_hotkey();
            }

            if self.hotkey_changed {
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("⚠  Restart KeySpeak to apply the new hotkey")
                        .color(egui::Color32::from_rgb(200, 140, 0))
                        .small(),
                );
            }

            ui.add_space(14.0);

            // ── Language ───────────────────────────────────────────────────
            ui.label(egui::RichText::new("🌐  Transcription Language").strong());
            ui.add_space(2.0);
            egui::ComboBox::from_id_source("language")
                .selected_text(LANGUAGES[self.lang_idx].1)
                .width(180.0)
                .show_ui(ui, |ui| {
                    for (i, &(code, name)) in LANGUAGES.iter().enumerate() {
                        if ui.selectable_value(&mut self.lang_idx, i, name).clicked() {
                            self.settings.language = code.to_string();
                        }
                    }
                });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(10.0);

            // ── Word Corrections ───────────────────────────────────────────
            ui.label(egui::RichText::new("✏️  Word Corrections").strong());
            ui.label(
                egui::RichText::new("Fix words KeySpeak consistently gets wrong")
                    .color(egui::Color32::GRAY)
                    .small(),
            );
            ui.add_space(6.0);

            // Column headers
            ui.horizontal(|ui| {
                ui.add_sized(
                    [200.0, 16.0],
                    egui::Label::new(egui::RichText::new("Hears").weak().small()),
                );
                ui.add_sized(
                    [200.0, 16.0],
                    egui::Label::new(egui::RichText::new("Should be").weak().small()),
                );
            });

            let mut remove: Option<usize> = None;
            egui::ScrollArea::vertical()
                .max_height(170.0)
                .show(ui, |ui| {
                    for (i, c) in self.settings.corrections.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [200.0, 22.0],
                                egui::TextEdit::singleline(&mut c.from).hint_text("e.g. clod"),
                            );
                            ui.add_sized(
                                [200.0, 22.0],
                                egui::TextEdit::singleline(&mut c.to).hint_text("e.g. Claude"),
                            );
                            if ui.small_button("✕").clicked() {
                                remove = Some(i);
                            }
                        });
                    }
                });
            if let Some(i) = remove {
                self.settings.corrections.remove(i);
            }

            // Add new correction row
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_sized(
                    [200.0, 22.0],
                    egui::TextEdit::singleline(&mut self.new_from).hint_text("Hears…"),
                );
                ui.add_sized(
                    [200.0, 22.0],
                    egui::TextEdit::singleline(&mut self.new_to).hint_text("Should be…"),
                );
                let ready = !self.new_from.is_empty() && !self.new_to.is_empty();
                ui.add_enabled_ui(ready, |ui| {
                    if ui.button("＋ Add").clicked() {
                        self.settings.corrections.push(Correction {
                            from: std::mem::take(&mut self.new_from),
                            to: std::mem::take(&mut self.new_to),
                        });
                    }
                });
            });

            ui.add_space(18.0);
            ui.separator();
            ui.add_space(10.0);

            // ── Save ───────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [120.0, 32.0],
                        egui::Button::new(egui::RichText::new("Save Settings").strong()),
                    )
                    .clicked()
                {
                    match self.settings.save() {
                        Ok(_) => {
                            self.status = Some("✅ Saved!".to_string());
                            self.pending_save = true;
                        }
                        Err(e) => {
                            self.status = Some(format!("❌ {}", e));
                        }
                    }
                }
                if let Some(ref msg) = self.status {
                    ui.label(msg);
                }
            });

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    "By using KeySpeak you agree to the terms at keyspeak.app/terms",
                )
                .color(egui::Color32::DARK_GRAY)
                .small(),
            );
        });
    }

    fn sync_hotkey(&mut self) {
        self.settings.hotkey.modifiers = match MODIFIERS[self.modifier_idx] {
            "Alt (⌥)" => vec!["Alt".to_string()],
            "Ctrl (⌃)" => vec!["Ctrl".to_string()],
            "Shift (⇧)" => vec!["Shift".to_string()],
            "Meta (⌘)" => vec!["Meta".to_string()],
            "Ctrl+Shift" => vec!["Ctrl".to_string(), "Shift".to_string()],
            _ => vec!["Alt".to_string()],
        };
        self.settings.hotkey.key = KEYS[self.key_idx].to_string();
    }
}

fn first_modifier(mods: &[String]) -> &str {
    mods.first().map(|s| s.as_str()).unwrap_or("Alt")
}
