# Changelog

All notable changes to KeySpeak are documented here.
Format: [Keep a Changelog](https://keepachangelog.com)
Versioning: [Semantic Versioning](https://semver.org)

## [Unreleased]

## [1.1.0] - 2026-03-18

### Added
- Native egui settings window (replaces browser-based HTML UI)
- Settings window opens directly below the tray icon, always on top
- Live settings reload - hotkey, language, and corrections apply without restarting
- Press Enter during recording to insert a newline in the transcription
- Custom KeySpeak icon in the menu bar with color states
- Noise filtering - strips Whisper hallucinations like [crickets chirping], (background noise)
- Common silence hallucination detection (e.g. "thank you", "thanks for watching")
- Global keyboard shortcuts: Ctrl+Option+Shift+, for Settings, Ctrl+Option+Shift+Q for Quit
- Version number shown in settings window

### Changed
- Upgraded eframe 0.27 to 0.29, egui 0.27 to 0.29, winit 0.29 to 0.30
- Settings window is fixed size (not resizable)
- Recording icon color toned down to a muted red
- Removed all emojis from settings UI

### Fixed
- macOS Apple Silicon crash caused by eframe 0.27 objc2 type mismatch in NSScreen enumeration

## [1.0.0] - 2025-06-01

### Added
- Global configurable hotkey (default ⌥ Space) works in any app
- Local transcription via whisper.cpp with Apple Metal GPU acceleration
- Waveform menu bar icon: grey (idle) · red (recording) · blue (transcribing)
- Real-time floating overlay showing words as you speak
- Settings window: hotkey picker, language selector, word corrections table
- Word corrections engine — case-insensitive whole-word replacement
- 14 language options including English, French, German, Spanish, Japanese
- Settings persisted to ~/Library/Preferences/keyspeak/settings.json
- Whisper small model (500 MB) for best accuracy/speed balance on Apple Silicon
- Full MIT licence — open source, no subscription, no cloud

[Unreleased]: https://github.com/markstent/keyspeak/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/markstent/keyspeak/releases/tag/v1.1.0
[1.0.0]: https://github.com/markstent/keyspeak/releases/tag/v1.0.0
