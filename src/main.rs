mod audio;
mod corrections;
mod icons;
#[allow(dead_code)] // overlay rendering planned but not yet wired to main event loop
mod overlay;
mod permissions;
mod settings;
#[allow(dead_code)] // used by the keyspeak-settings binary
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
    event_loop::{ActiveEventLoop, ControlFlow},
};

#[derive(Debug)]
enum AppMsg {
    TranscriptionDone(String),
    TranscriptionFailed(String),
    OpenSettings,
    ReloadSettings,
}

fn main() -> Result<()> {
    // Redirect stderr to a log file so we can diagnose when launched from Finder
    let log_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("keyspeak")
        .join("keyspeak.log");
    if let Ok(file) = std::fs::File::create(&log_path) {
        use std::os::unix::io::IntoRawFd;
        let fd = file.into_raw_fd();
        unsafe {
            libc::dup2(fd, 2);
        }
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    eprintln!("[main] KeySpeak starting, log: {}", log_path.display());
    permissions::ensure_permissions();

    let settings = Arc::new(Mutex::new(Settings::load()));

    // ── Overlay state (logged to terminal — macOS requires GUI on main thread,
    //    which is occupied by the winit event loop for tray/hotkey) ──────────
    let overlay_state = Arc::new(Mutex::new(overlay::OverlayState::default()));

    // ── Main event loop ───────────────────────────────────────────────────
    let event_loop = winit::event_loop::EventLoop::<AppMsg>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    // ── Tray icon + menu ──────────────────────────────────────────────────
    use tray_icon::menu::accelerator::{Accelerator, Code as AccelCode, Modifiers as AccelMods};
    let settings_accel = Accelerator::new(
        Some(AccelMods::CONTROL | AccelMods::ALT | AccelMods::SHIFT),
        AccelCode::Comma,
    );
    let quit_accel = Accelerator::new(
        Some(AccelMods::CONTROL | AccelMods::ALT | AccelMods::SHIFT),
        AccelCode::KeyQ,
    );
    let settings_item = MenuItem::new("Settings", true, Some(settings_accel));
    let quit_item = MenuItem::new("Quit KeySpeak", true, Some(quit_accel));
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

    let current_hotkey = Arc::new(Mutex::new(hotkey));

    // Global shortcuts for Settings and Quit
    let settings_hotkey = HotKey::new(
        Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
        Code::Comma,
    );
    let settings_hk_id = settings_hotkey.id();
    hk_manager.register(settings_hotkey)?;

    let quit_hotkey = HotKey::new(
        Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
        Code::KeyQ,
    );
    let quit_hk_id = quit_hotkey.id();
    hk_manager.register(quit_hotkey)?;

    println!(
        "KeySpeak ready - {}",
        settings.lock().unwrap().hotkey.display()
    );

    // ── Settings file watcher (polls for changes) ──────────────────────
    {
        let proxy_w = proxy.clone();
        thread::spawn(move || {
            let path = Settings::config_path();
            let mut last_modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
            loop {
                thread::sleep(Duration::from_secs(2));
                let current = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                if current != last_modified {
                    last_modified = current;
                    let _ = proxy_w.send_event(AppMsg::ReloadSettings);
                }
            }
        });
    }

    // ── Enter key for newlines during recording ─────────────────────────
    let enter_hotkey = HotKey::new(None, Code::Enter);
    let enter_id = enter_hotkey.id();

    // ── Shared recording state ────────────────────────────────────────────
    let is_recording = Arc::new(Mutex::new(false));
    let rec_handle: Arc<Mutex<Option<audio::RecordingHandle>>> = Arc::new(Mutex::new(None));
    let sample_buf = Arc::new(Mutex::new(Vec::<f32>::new()));
    let sample_rate = Arc::new(Mutex::new(44_100u32));
    let newline_positions: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));

    // ── Event loop ────────────────────────────────────────────────────────
    #[allow(deprecated)]
    event_loop.run(move |event: Event<AppMsg>, elwt: &ActiveEventLoop| {
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
                    open_settings_window(settings.lock().unwrap().clone(), tray.rect());
                }

                AppMsg::ReloadSettings => {
                    let new_settings = Settings::load();
                    // Re-register hotkey if it changed
                    if let Ok(new_hk) = build_hotkey(&new_settings) {
                        let old_hk = *current_hotkey.lock().unwrap();
                        if new_hk != old_hk {
                            let _ = hk_manager.unregister(old_hk);
                            if hk_manager.register(new_hk).is_ok() {
                                *current_hotkey.lock().unwrap() = new_hk;
                                eprintln!(
                                    "[info] hotkey changed to {}",
                                    new_settings.hotkey.display()
                                );
                            }
                        }
                    }
                    tray.set_tooltip(Some(format!(
                        "KeySpeak - {}",
                        new_settings.hotkey.display(),
                    )))
                    .ok();
                    *settings.lock().unwrap() = new_settings;
                }
            },

            Event::AboutToWait => {
                // ── Hotkey ─────────────────────────────────────────────────
                if let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv() {
                    use global_hotkey::HotKeyState;
                    if ev.state == HotKeyState::Pressed {
                        // Settings shortcut
                        if ev.id == settings_hk_id {
                            let _ = proxy.send_event(AppMsg::OpenSettings);
                        // Quit shortcut
                        } else if ev.id == quit_hk_id {
                            elwt.exit();
                        // Enter pressed during recording -> mark newline
                        } else if ev.id == enter_id {
                            if *is_recording.lock().unwrap() {
                                let pos = sample_buf.lock().unwrap().len();
                                newline_positions.lock().unwrap().push(pos);
                                eprintln!("[info] newline marked at sample {}", pos);
                            }
                        } else {
                            // Main hotkey -> toggle recording
                            let mut rec = is_recording.lock().unwrap();

                            if *rec {
                                // STOP ─────────────────────────────────────
                                *rec = false;
                                drop(rec);

                                // Unregister Enter hotkey
                                let _ = hk_manager.unregister(enter_hotkey);

                                let handle = rec_handle.lock().unwrap().take();
                                let sr = *sample_rate.lock().unwrap();
                                drop(handle); // stops the audio stream

                                {
                                    let mut ov = overlay_state.lock().unwrap();
                                    ov.is_processing = true;
                                }
                                tray.set_icon(Some(icons::thinking_icon())).ok();
                                tray.set_tooltip(Some("KeySpeak - Transcribing...")).ok();

                                let samples = std::mem::take(&mut *sample_buf.lock().unwrap());
                                let nl_pos =
                                    std::mem::take(&mut *newline_positions.lock().unwrap());
                                let s = settings.lock().unwrap();
                                let model = s.model_path.clone();
                                let lang = s.language.clone();
                                let corrections = s.corrections.clone();
                                drop(s);
                                let proxy_c = proxy.clone();

                                thread::spawn(move || {
                                    let peak =
                                        samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                                    let rms = (samples.iter().map(|s| s * s).sum::<f32>()
                                        / samples.len().max(1) as f32)
                                        .sqrt();
                                    eprintln!(
                                        "[info] {} samples at {}Hz, peak={:.4}, rms={:.4}",
                                        samples.len(),
                                        sr,
                                        peak,
                                        rms
                                    );

                                    let text = if nl_pos.is_empty() {
                                        // No newlines - transcribe as one chunk
                                        let s16k = audio::resample_to_16k(samples, sr);
                                        eprintln!("[info] resampled to {} samples", s16k.len());
                                        transcribe::transcribe(&s16k, &model, &lang)
                                    } else {
                                        // Split at newline positions, transcribe each chunk
                                        transcribe_with_newlines(
                                            &samples, &nl_pos, sr, &model, &lang,
                                        )
                                    };

                                    match text {
                                        Ok(raw) => {
                                            let fixed = corrections::apply(&raw, &corrections);
                                            let _ = proxy_c
                                                .send_event(AppMsg::TranscriptionDone(fixed));
                                        }
                                        Err(e) => {
                                            let _ = proxy_c.send_event(
                                                AppMsg::TranscriptionFailed(e.to_string()),
                                            );
                                        }
                                    }
                                });
                            } else {
                                // START ────────────────────────────────────
                                *rec = true;
                                drop(rec);
                                sample_buf.lock().unwrap().clear();
                                newline_positions.lock().unwrap().clear();

                                // Register Enter hotkey for newlines
                                let _ = hk_manager.register(enter_hotkey);

                                {
                                    let mut ov = overlay_state.lock().unwrap();
                                    ov.visible = true;
                                    ov.is_processing = false;
                                    ov.text = String::new();
                                }
                                tray.set_icon(Some(icons::recording_icon())).ok();
                                tray.set_tooltip(Some("KeySpeak - Recording...")).ok();

                                let buf_c = sample_buf.clone();
                                let (tx, rx) = unbounded::<Vec<f32>>();

                                // Audio drain thread - collects samples into buffer
                                thread::spawn(move || {
                                    while let Ok(chunk) = rx.recv() {
                                        buf_c.lock().unwrap().extend(chunk);
                                    }
                                });

                                match audio::start_recording(tx) {
                                    Ok(h) => {
                                        *sample_rate.lock().unwrap() = h.device_sample_rate;
                                        *rec_handle.lock().unwrap() = Some(h);
                                    }
                                    Err(e) => {
                                        eprintln!("Mic error: {}", e);
                                        overlay_state.lock().unwrap().visible = false;
                                        *is_recording.lock().unwrap() = false;
                                        let _ = hk_manager.unregister(enter_hotkey);
                                    }
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

fn open_settings_window(current: Settings, tray_rect: Option<tray_icon::Rect>) {
    let exe = std::env::current_exe().unwrap_or_default();
    let settings_bin = exe.with_file_name("keyspeak-settings");

    let mut cmd = if settings_bin.exists() {
        std::process::Command::new(&settings_bin)
    } else {
        std::process::Command::new("keyspeak-settings")
    };

    // Pass tray icon position so the window opens directly below it
    if let Some(rect) = tray_rect {
        cmd.arg("--tray-x")
            .arg(format!("{}", rect.position.x))
            .arg("--tray-y")
            .arg(format!("{}", rect.position.y + rect.size.height as f64));
    }

    if cmd.spawn().is_err() {
        // Fallback: open JSON in default editor
        let _ = current.save();
        let _ = std::process::Command::new("open")
            .arg(Settings::config_path())
            .spawn();
    }
}

fn transcribe_with_newlines(
    samples: &[f32],
    nl_pos: &[usize],
    sr: u32,
    model: &str,
    lang: &str,
) -> Result<String> {
    let mut chunks: Vec<&[f32]> = Vec::new();
    let mut start = 0;
    for &pos in nl_pos {
        let pos = pos.min(samples.len());
        if pos > start {
            chunks.push(&samples[start..pos]);
        }
        start = pos;
    }
    if start < samples.len() {
        chunks.push(&samples[start..]);
    }

    let mut lines = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let s16k = audio::resample_to_16k(chunk.to_vec(), sr);
        eprintln!(
            "[info] chunk {}: {} -> {} samples",
            i,
            chunk.len(),
            s16k.len()
        );
        match transcribe::transcribe(&s16k, model, lang) {
            Ok(text) if !text.is_empty() => lines.push(text),
            Ok(_) => {} // empty chunk, skip
            Err(e) => eprintln!("[warn] chunk {} failed: {}", i, e),
        }
    }
    Ok(lines.join("\n"))
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
