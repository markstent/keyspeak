//! egui settings window - hotkey, language, word corrections.

use crate::settings::{Correction, Settings};
use eframe::egui;

const MODIFIERS: &[&str] = &["Alt", "Ctrl", "Shift", "Meta", "Ctrl+Shift"];

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

// macOS-style dark palette
const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(28, 28, 30);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(44, 44, 46);
const INPUT_BG: egui::Color32 = egui::Color32::from_rgb(36, 36, 38);
const BORDER: egui::Color32 = egui::Color32::from_rgb(58, 58, 60);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(245, 245, 250);
const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(190, 190, 195);
const TEXT_TERTIARY: egui::Color32 = egui::Color32::from_rgb(120, 120, 125);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(10, 132, 255);
const SUCCESS: egui::Color32 = egui::Color32::from_rgb(48, 209, 88);
const ERROR: egui::Color32 = egui::Color32::from_rgb(255, 69, 58);

// Spacing constants for consistent rhythm
const MARGIN: f32 = 20.0;
const CARD_PAD: f32 = 14.0;
const CARD_GAP: f32 = 10.0;
const FIELD_HEIGHT: f32 = 28.0;
const INNER_GAP: f32 = 8.0;

pub struct SettingsWindow {
    pub settings: Settings,
    pub pending_save: bool,
    new_from: String,
    new_to: String,
    modifier_idx: usize,
    key_idx: usize,
    lang_idx: usize,
    status: Option<(String, egui::Color32)>,
}

impl SettingsWindow {
    pub fn new(settings: Settings) -> Self {
        let modifier_idx = MODIFIERS
            .iter()
            .position(|&m| m == first_modifier(&settings.hotkey.modifiers))
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
            new_from: String::new(),
            new_to: String::new(),
            modifier_idx,
            key_idx,
            lang_idx,
            status: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.apply_style(ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(PANEL_BG)
                    .inner_margin(egui::Margin::same(MARGIN)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
                let panel_w = ui.available_width();

                // Title + version
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Settings")
                            .size(18.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .size(11.0)
                                .color(TEXT_TERTIARY),
                        );
                    });
                });
                ui.add_space(CARD_GAP);

                // -- Hotkey & Language in a single card --
                self.card(ui, panel_w, |ui, this| {
                    let inner_w = ui.available_width();

                    // Hotkey row
                    ui.label(
                        egui::RichText::new("Record Hotkey")
                            .size(11.0)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(2.0);

                    let prev = (this.modifier_idx, this.key_idx);
                    ui.horizontal(|ui| {
                        let combo_w = (inner_w - 30.0) / 2.0;
                        egui::ComboBox::from_id_salt("modifier")
                            .selected_text(MODIFIERS[this.modifier_idx])
                            .width(combo_w)
                            .show_ui(ui, |ui| {
                                for (i, &m) in MODIFIERS.iter().enumerate() {
                                    ui.selectable_value(&mut this.modifier_idx, i, m);
                                }
                            });
                        ui.label(egui::RichText::new("+").size(13.0).color(TEXT_SECONDARY));
                        egui::ComboBox::from_id_salt("key")
                            .selected_text(KEYS[this.key_idx])
                            .width(combo_w)
                            .show_ui(ui, |ui| {
                                for (i, &k) in KEYS.iter().enumerate() {
                                    ui.selectable_value(&mut this.key_idx, i, k);
                                }
                            });
                    });
                    if (this.modifier_idx, this.key_idx) != prev {
                        this.sync_hotkey();
                    }

                    ui.add_space(INNER_GAP + 4.0);

                    // Thin separator
                    let rect = ui.available_rect_before_wrap();
                    ui.painter()
                        .hline(rect.x_range(), rect.top(), egui::Stroke::new(0.5, BORDER));
                    ui.add_space(INNER_GAP + 4.0);

                    // Language row
                    ui.label(
                        egui::RichText::new("Language")
                            .size(11.0)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(2.0);
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(LANGUAGES[this.lang_idx].1)
                        .width(inner_w)
                        .show_ui(ui, |ui| {
                            for (i, &(code, name)) in LANGUAGES.iter().enumerate() {
                                if ui.selectable_value(&mut this.lang_idx, i, name).clicked() {
                                    this.settings.language = code.to_string();
                                }
                            }
                        });
                });

                ui.add_space(CARD_GAP);

                // -- Word Corrections card --
                self.card(ui, panel_w, |ui, this| {
                    let inner_w = ui.available_width();

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Word Corrections")
                                .size(11.0)
                                .color(TEXT_SECONDARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("Fix words that are consistently misheard")
                                    .size(10.0)
                                    .color(TEXT_TERTIARY),
                            );
                        });
                    });
                    ui.add_space(INNER_GAP);

                    // Column headers
                    let col_w = (inner_w - INNER_GAP) / 2.0;
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [col_w, 12.0],
                            egui::Label::new(
                                egui::RichText::new("HEARS").size(10.0).color(TEXT_TERTIARY),
                            ),
                        );
                        ui.add_sized(
                            [col_w, 12.0],
                            egui::Label::new(
                                egui::RichText::new("REPLACE WITH")
                                    .size(10.0)
                                    .color(TEXT_TERTIARY),
                            ),
                        );
                    });
                    ui.add_space(2.0);

                    // Existing corrections
                    let mut remove: Option<usize> = None;
                    let remove_w = 20.0;
                    let field_w = (inner_w - INNER_GAP - remove_w - 4.0) / 2.0;

                    egui::ScrollArea::vertical()
                        .max_height(120.0)
                        .show(ui, |ui| {
                            for (i, c) in this.settings.corrections.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.add_sized(
                                        [field_w, FIELD_HEIGHT],
                                        egui::TextEdit::singleline(&mut c.from)
                                            .hint_text("e.g. clod"),
                                    );
                                    ui.add_sized(
                                        [field_w, FIELD_HEIGHT],
                                        egui::TextEdit::singleline(&mut c.to)
                                            .hint_text("e.g. Claude"),
                                    );
                                    if ui
                                        .add_sized(
                                            [remove_w, FIELD_HEIGHT],
                                            egui::Button::new(
                                                egui::RichText::new("x")
                                                    .size(12.0)
                                                    .color(TEXT_SECONDARY),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                    {
                                        remove = Some(i);
                                    }
                                });
                            }
                        });
                    if let Some(i) = remove {
                        this.settings.corrections.remove(i);
                    }

                    ui.add_space(4.0);

                    // Add new correction - inputs + button all in one row
                    let add_w = 44.0;
                    let add_field_w = (inner_w - INNER_GAP - add_w - 4.0) / 2.0;
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [add_field_w, FIELD_HEIGHT],
                            egui::TextEdit::singleline(&mut this.new_from).hint_text("Hears..."),
                        );
                        ui.add_sized(
                            [add_field_w, FIELD_HEIGHT],
                            egui::TextEdit::singleline(&mut this.new_to).hint_text("Should be..."),
                        );
                        let ready = !this.new_from.is_empty() && !this.new_to.is_empty();
                        let btn = egui::Button::new(
                            egui::RichText::new("Add").size(12.0).color(if ready {
                                ACCENT
                            } else {
                                TEXT_TERTIARY
                            }),
                        );
                        if ui.add_sized([add_w, FIELD_HEIGHT], btn).clicked() && ready {
                            this.settings.corrections.push(Correction {
                                from: std::mem::take(&mut this.new_from),
                                to: std::mem::take(&mut this.new_to),
                            });
                        }
                    });
                });

                ui.add_space(CARD_GAP);

                // Save button - right aligned, compact
                ui.horizontal(|ui| {
                    if let Some((ref msg, color)) = self.status {
                        ui.label(egui::RichText::new(msg).color(color).size(12.0));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let save_btn = egui::Button::new(
                            egui::RichText::new("Save")
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(ACCENT)
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(72.0, 30.0));

                        if ui.add(save_btn).clicked() {
                            match self.settings.save() {
                                Ok(_) => {
                                    self.status = Some(("Saved".to_string(), SUCCESS));
                                    self.pending_save = true;
                                }
                                Err(e) => {
                                    self.status = Some((e.to_string(), ERROR));
                                }
                            }
                        }
                    });
                });

                ui.add_space(INNER_GAP);
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "By using KeySpeak you agree to the terms at keyspeak.app/terms",
                        )
                        .size(10.0)
                        .color(TEXT_TERTIARY),
                    );
                });
            });
    }

    fn card(
        &mut self,
        ui: &mut egui::Ui,
        width: f32,
        content: impl FnOnce(&mut egui::Ui, &mut Self),
    ) {
        egui::Frame::none()
            .fill(CARD_BG)
            .rounding(egui::Rounding::same(10.0))
            .stroke(egui::Stroke::new(0.5, BORDER))
            .inner_margin(egui::Margin::same(CARD_PAD))
            .show(ui, |ui| {
                ui.set_width(width - CARD_PAD * 2.0);
                content(ui, self);
            });
    }

    fn sync_hotkey(&mut self) {
        self.settings.hotkey.modifiers = match MODIFIERS[self.modifier_idx] {
            "Alt" => vec!["Alt".to_string()],
            "Ctrl" => vec!["Ctrl".to_string()],
            "Shift" => vec!["Shift".to_string()],
            "Meta" => vec!["Meta".to_string()],
            "Ctrl+Shift" => vec!["Ctrl".to_string(), "Shift".to_string()],
            _ => vec!["Alt".to_string()],
        };
        self.settings.hotkey.key = KEYS[self.key_idx].to_string();
    }

    fn apply_style(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Consistent rounding
        let rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.inactive.rounding = rounding;
        style.visuals.widgets.hovered.rounding = rounding;
        style.visuals.widgets.active.rounding = rounding;
        style.visuals.widgets.noninteractive.rounding = rounding;

        // Input styling
        style.visuals.widgets.inactive.bg_fill = INPUT_BG;
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(0.5, BORDER);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);

        // Text colors
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);

        // Selection
        style.visuals.selection.bg_fill = ACCENT;

        // Popups / dropdowns
        style.visuals.window_fill = CARD_BG;
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.window_stroke = egui::Stroke::new(0.5, BORDER);
        style.visuals.widgets.noninteractive.bg_fill = CARD_BG;

        // Tighten global spacing
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);

        ctx.set_style(style);
    }
}

fn first_modifier(mods: &[String]) -> &str {
    if mods.len() > 1 {
        "Ctrl+Shift"
    } else {
        mods.first().map(|s| s.as_str()).unwrap_or("Alt")
    }
}
