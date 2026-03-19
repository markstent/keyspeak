<div align="center">

# KeySpeak

**Local speech-to-text for macOS. Press a key, speak, text appears.**

[![CI](https://github.com/markstent/keyspeak/actions/workflows/ci.yml/badge.svg)](https://github.com/markstent/keyspeak/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![macOS](https://img.shields.io/badge/macOS-13%2B-black?logo=apple)](https://github.com/markstent/keyspeak/releases)

[Website](https://keyspeak.shop) ·
[Buy ready-made DMG — $5](https://keyspeak.shop/#pricing)

</div>

KeySpeak puts a waveform icon in your menu bar. Press your chosen shortcut
anywhere on your Mac — in any app — speak, press it again, and your words
are typed at the cursor.

Everything runs locally using [Whisper](https://github.com/ggerganov/whisper.cpp).
Your audio never leaves your machine.

## Features

- 🎙 **Works everywhere** — any app that accepts text input
- 🔒 **Fully private** — no internet, no API keys, no servers
- ⚡ **Fast** — Apple Silicon Metal acceleration via whisper.cpp
- ⌨️ **Configurable hotkey** — any modifier + key combination
- 👁 **Live preview** — see your words as you speak
- ✏️ **Word corrections** — fix names and jargon Whisper gets wrong
- 🌐 **14 languages** supported

## Requirements

- macOS 13 (Ventura) or later
- Apple Silicon (M1/M2/M3/M4) recommended
- ~500 MB disk space for the Whisper model

## Installation

**Option 1 — Buy the compiled app:** Get the ready-to-use DMG at
[keyspeak.shop](https://keyspeak.shop/#pricing) ($5, one-time).

**Option 2 — Build from source (free):**
```bash
# Install prerequisites
brew install cmake
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Download the Whisper model
mkdir -p ~/Library/Application\ Support/keyspeak
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin" \
     -o ~/Library/Application\ Support/keyspeak/ggml-small.bin

# Build and run
cargo run
```

## Permissions

| Permission | Why |
|---|---|
| Microphone | To record your voice |
| Accessibility | To type transcribed text into other apps |

## Privacy

All audio is processed locally. No data leaves your machine.
No telemetry, no analytics, no network requests.

## Licence

MIT — see [LICENSE](LICENSE).
Purchase a compiled release at [keyspeak.shop](https://keyspeak.shop).
