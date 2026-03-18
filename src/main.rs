mod audio;
mod corrections;
mod icons;
mod overlay;
mod settings;
mod settings_ui;
mod transcribe;
mod typer;

use anyhow::Result;
use crossbeam_channel::unbounded;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use settings::Settings;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};

#[derive(Debug)]
enum AppMsg {
    TranscriptionDone(String),
    TranscriptionFailed(String),
    OpenSettings,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let settings = Arc::new(Mutex::new(Settings::load()));

    // ── Floating overlay (own thread + egui event loop) ───────────────────
    let overlay_state = Arc::new(Mutex::new(overlay::OverlayState::default()));
    {
        let ov = overlay_state.clone();
        thread::spawn(move || {
            let opts = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_decorations(false)
                    .with_transparent(true)
                    .with_always_on_top()
                    .with_inner_size([540.0, 52.0])
                    .with_position([380.0, 900.0])
                    .with_resizable(false),
                ..Default::default()
            };
            let _ = eframe::run_native(
                "KeySpeak Overlay",
                opts,
                Box::new(|_cc| Box::new(overlay::OverlayApp { state: ov })),
            );
        });
    }

    // ── Main event loop ───────────────────────────────────────────────────
    let event_loop = EventLoopBuilder::<AppMsg>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    // ── Tray icon + menu ──────────────────────────────────────────────────
    let settings_item = MenuItem::new("Settings…", true, None);
    let quit_item = MenuItem::new("Quit KeySpeak", true, None);
    let menu = Menu::new();
    menu.append(&settings_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let tooltip = format!("KeySpeak — {}", settings.lock().unwrap().hotkey.display());
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(icons::idle_icon()) // waveform icon, grey
        .with_tooltip(tooltip)
        .build()?;

    // ── Global hotkey ─────────────────────────────────────────────────────
    let hk_manager = GlobalHotKeyManager::new()?;
    let hotkey = build_hotkey(&settings.lock().unwrap())?;
    hk_manager.register(hotkey)?;

    println!(
        "✅ KeySpeak — {}",
        settings.lock().unwrap().hotkey.display()
    );

    // ── Shared recording state ────────────────────────────────────────────
    let is_recording = Arc::new(Mutex::new(false));
    let rec_handle: Arc<Mutex<Option<audio::RecordingHandle>>> = Arc::new(Mutex::new(None));
    let sample_buf = Arc::new(Mutex::new(Vec::<f32>::new()));
    let sample_rate = Arc::new(Mutex::new(44_100u32));

    // ── Event loop ────────────────────────────────────────────────────────
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::UserEvent(msg) => match msg {
                AppMsg::TranscriptionDone(text) => {
                    overlay_state.lock().unwrap().visible = false;
                    tray.set_icon(Some(icons::idle_icon())).ok();
                    tray.set_tooltip(Some(format!(
                        "KeySpeak — {}",
                        settings.lock().unwrap().hotkey.display(),
                    )))
                    .ok();

                    if !text.is_empty() {
                        // Small delay so the hotkey-release event doesn't
                        // land in the target app before our text does
                        thread::sleep(Duration::from_millis(150));
                        if let Err(e) = typer::type_at_cursor(&text) {
                            eprintln!("⚠️  {}", e);
                        }
                    }
                }

                AppMsg::TranscriptionFailed(err) => {
                    overlay_state.lock().unwrap().visible = false;
                    tray.set_icon(Some(icons::idle_icon())).ok();
                    eprintln!("❌ {}", err);
                }

                AppMsg::OpenSettings => {
                    open_settings_window(settings.lock().unwrap().clone());
                }
            },

            Event::AboutToWait => {
                // ── Hotkey ─────────────────────────────────────────────────
                if let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv() {
                    use global_hotkey::HotKeyState;
                    if ev.state == HotKeyState::Pressed {
                        let mut rec = is_recording.lock().unwrap();

                        if *rec {
                            // STOP ─────────────────────────────────────────
                            *rec = false;
                            drop(rec);

                            let handle = rec_handle.lock().unwrap().take();
                            let sr = *sample_rate.lock().unwrap();
                            drop(handle); // stops the audio stream

                            {
                                let mut ov = overlay_state.lock().unwrap();
                                ov.is_processing = true;
                            }
                            tray.set_icon(Some(icons::thinking_icon())).ok();
                            tray.set_tooltip(Some("KeySpeak — Transcribing…")).ok();

                            let samples = std::mem::take(&mut *sample_buf.lock().unwrap());
                            let model = settings.lock().unwrap().model_path.clone();
                            let lang = settings.lock().unwrap().language.clone();
                            let corrections = settings.lock().unwrap().corrections.clone();
                            let proxy_c = proxy.clone();

                            thread::spawn(move || {
                                let s16k = audio::resample_to_16k(samples, sr);
                                match transcribe::transcribe(&s16k, &model, &lang) {
                                    Ok(raw) => {
                                        let fixed = corrections::apply(&raw, &corrections);
                                        let _ =
                                            proxy_c.send_event(AppMsg::TranscriptionDone(fixed));
                                    }
                                    Err(e) => {
                                        let _ = proxy_c
                                            .send_event(AppMsg::TranscriptionFailed(e.to_string()));
                                    }
                                }
                            });
                        } else {
                            // START ────────────────────────────────────────
                            *rec = true;
                            drop(rec);
                            sample_buf.lock().unwrap().clear();

                            {
                                let mut ov = overlay_state.lock().unwrap();
                                ov.visible = true;
                                ov.is_processing = false;
                                ov.text = String::new();
                            }
                            tray.set_icon(Some(icons::recording_icon())).ok();
                            tray.set_tooltip(Some("KeySpeak — Recording…")).ok();

                            let buf_c = sample_buf.clone();
                            let ov_c = overlay_state.clone();
                            let model = settings.lock().unwrap().model_path.clone();
                            let lang = settings.lock().unwrap().language.clone();
                            let (tx, rx) = unbounded::<Vec<f32>>();

                            // Audio drain + rolling partial transcript
                            thread::spawn(move || {
                                let mut all: Vec<f32> = Vec::new();
                                let mut last_partial: std::time::Instant =
                                    std::time::Instant::now();

                                while let Ok(chunk) = rx.recv() {
                                    all.extend(chunk.clone());
                                    buf_c.lock().unwrap().extend(chunk);

                                    if last_partial.elapsed().as_millis() > 1_500
                                        && all.len() > 16_000
                                    {
                                        let ps = all.clone();
                                        let mc = model.clone();
                                        let lc = lang.clone();
                                        let oc = ov_c.clone();
                                        thread::spawn(move || {
                                            if let Ok(p) = transcribe::transcribe(&ps, &mc, &lc) {
                                                if !p.is_empty() {
                                                    oc.lock().unwrap().text = p;
                                                }
                                            }
                                        });
                                        last_partial = std::time::Instant::now();
                                    }
                                }
                            });

                            match audio::start_recording(tx) {
                                Ok(h) => {
                                    *sample_rate.lock().unwrap() = h.device_sample_rate;
                                    *rec_handle.lock().unwrap() = Some(h);
                                }
                                Err(e) => {
                                    eprintln!("❌ Mic error: {}", e);
                                    overlay_state.lock().unwrap().visible = false;
                                    *is_recording.lock().unwrap() = false;
                                }
                            }
                        }
                    }
                }

                // ── Tray menu ──────────────────────────────────────────────
                if let Ok(ev) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                    if ev.id == quit_item.id() {
                        elwt.exit();
                    }
                    if ev.id == settings_item.id() {
                        let _ = proxy.send_event(AppMsg::OpenSettings);
                    }
                }
            }

            _ => {}
        }
    })?;

    Ok(())
}

fn open_settings_window(current: Settings) {
    thread::spawn(move || {
        let opts = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("KeySpeak Settings")
                .with_inner_size([520.0, 560.0])
                .with_resizable(false),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "KeySpeak Settings",
            opts,
            Box::new(move |_cc| {
                Box::new(SettingsApp {
                    ui: settings_ui::SettingsWindow::new(current),
                })
            }),
        );
    });
}

struct SettingsApp {
    ui: settings_ui::SettingsWindow,
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui.show(ctx);
        if self.ui.pending_save {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn build_hotkey(s: &Settings) -> Result<HotKey> {
    let mut mods = Modifiers::empty();
    for m in &s.hotkey.modifiers {
        match m.as_str() {
            "Alt" => mods |= Modifiers::ALT,
            "Ctrl" => mods |= Modifiers::CONTROL,
            "Shift" => mods |= Modifiers::SHIFT,
            "Meta" => mods |= Modifiers::META,
            other => eprintln!("⚠️  Unknown modifier '{}' — skipping", other),
        }
    }
    Ok(HotKey::new(Some(mods), parse_key(&s.hotkey.key)?))
}

fn parse_key(key: &str) -> Result<Code> {
    match key {
        "Space" => Ok(Code::Space),
        "KeyA" => Ok(Code::KeyA),
        "KeyB" => Ok(Code::KeyB),
        "KeyC" => Ok(Code::KeyC),
        "KeyD" => Ok(Code::KeyD),
        "KeyE" => Ok(Code::KeyE),
        "KeyF" => Ok(Code::KeyF),
        "KeyG" => Ok(Code::KeyG),
        "KeyH" => Ok(Code::KeyH),
        "KeyI" => Ok(Code::KeyI),
        "KeyJ" => Ok(Code::KeyJ),
        "KeyK" => Ok(Code::KeyK),
        "KeyL" => Ok(Code::KeyL),
        "KeyM" => Ok(Code::KeyM),
        "KeyN" => Ok(Code::KeyN),
        "KeyO" => Ok(Code::KeyO),
        "KeyP" => Ok(Code::KeyP),
        "KeyQ" => Ok(Code::KeyQ),
        "KeyR" => Ok(Code::KeyR),
        "KeyS" => Ok(Code::KeyS),
        "KeyT" => Ok(Code::KeyT),
        "KeyU" => Ok(Code::KeyU),
        "KeyV" => Ok(Code::KeyV),
        "KeyW" => Ok(Code::KeyW),
        "KeyX" => Ok(Code::KeyX),
        "KeyY" => Ok(Code::KeyY),
        "KeyZ" => Ok(Code::KeyZ),
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),
        other => Err(anyhow::anyhow!(
            "Unknown key '{}'. Valid: Space, KeyA–Z, F1–F12",
            other
        )),
    }
}
