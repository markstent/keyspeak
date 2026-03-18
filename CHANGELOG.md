# Changelog

All notable changes to KeySpeak are documented here.
Format: [Keep a Changelog](https://keepachangelog.com)
Versioning: [Semantic Versioning](https://semver.org)

## [Unreleased]

## [1.0.0] — 2025-XX-XX

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

[1.0.0]: https://github.com/markstent/keyspeak/releases/tag/v1.0.0
