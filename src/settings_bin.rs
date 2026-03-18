use eframe::egui;
use keyspeak::settings::Settings;
use keyspeak::settings_ui::SettingsWindow;

struct SettingsApp {
    ui: SettingsWindow,
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui.show(ctx);
        if self.ui.pending_save {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn main() -> eframe::Result {
    let settings = Settings::load();

    let width = 420.0_f32;
    let height = 480.0_f32;
    let (tray_x, tray_y) = parse_tray_position();

    // Center the window horizontally on the tray icon, place it just below
    let x = tray_x - width / 2.0;
    let y = tray_y + 4.0;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_position(egui::pos2(x, y))
            .with_resizable(false)
            .with_title("KeySpeak Settings")
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "KeySpeak Settings",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(SettingsApp {
                ui: SettingsWindow::new(settings),
            }))
        }),
    )
}

/// Parse --tray-x and --tray-y from command-line args.
/// Falls back to top-right of screen if not provided.
fn parse_tray_position() -> (f32, f32) {
    let args: Vec<String> = std::env::args().collect();
    let mut x: Option<f32> = None;
    let mut y: Option<f32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tray-x" if i + 1 < args.len() => {
                x = args[i + 1].parse().ok();
                i += 2;
            }
            "--tray-y" if i + 1 < args.len() => {
                y = args[i + 1].parse().ok();
                i += 2;
            }
            _ => i += 1,
        }
    }

    match (x, y) {
        (Some(x), Some(y)) => (x, y),
        _ => {
            // Fallback: top-right area
            let screen_w = screen_width();
            (screen_w - 300.0, 30.0)
        }
    }
}

fn screen_width() -> f32 {
    std::process::Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
        .ok()
        .and_then(|out| {
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines()
                .find(|l| l.contains("Resolution") || l.contains("_resolution"))
                .and_then(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts.iter().find_map(|p| {
                        p.trim_matches(|c: char| !c.is_ascii_digit())
                            .parse::<f32>()
                            .ok()
                            .filter(|&v| v > 800.0)
                    })
                })
        })
        .unwrap_or(1440.0)
}
