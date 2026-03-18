//! Floating transparent overlay window that shows live transcription text.
//! Appears bottom-centre of screen while recording; disappears after typing.

use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
pub struct OverlayState {
    pub visible: bool,
    pub text: String,
    pub is_processing: bool,
}

pub struct OverlayApp {
    pub state: Arc<Mutex<OverlayState>>,
}

impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let state = self.state.lock().unwrap().clone();

        // Keep animating even when text hasn't changed (for the pulsing dot)
        ctx.request_repaint_after(std::time::Duration::from_millis(50));

        if !state.visible {
            return;
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(18, 18, 18, 230))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(16.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Pulse the dot in and out smoothly
                    let t = ctx.input(|i| i.time);
                    let pulse = ((t * 2.0).sin() * 0.28 + 0.72) as f32;

                    let [r, g, b]: [f32; 3] = if state.is_processing {
                        [48.0, 120.0, 210.0]
                    } else {
                        [210.0, 48.0, 48.0]
                    };

                    let dot = egui::Color32::from_rgba_unmultiplied(
                        (r * pulse) as u8,
                        (g * pulse) as u8,
                        (b * pulse) as u8,
                        255,
                    );

                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 5.0, dot);
                    ui.add_space(6.0);

                    let display = if state.is_processing {
                        "Transcribing…".into()
                    } else if state.text.is_empty() {
                        "Listening…".into()
                    } else {
                        state.text.clone()
                    };

                    ui.label(
                        egui::RichText::new(&display)
                            .color(egui::Color32::WHITE)
                            .size(15.0),
                    );
                });
            });
    }
}
