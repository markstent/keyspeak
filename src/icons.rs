//! Menu bar icons using the KeySpeak logo, tinted per state.
//!
//! The icon is a 32x32 RGBA image (suitable for Retina menu bars at 16pt @2x).
//! Three tinted variants:
//!   idle_icon()      -> white  (standard menu bar template style)
//!   recording_icon() -> red    (microphone is open)
//!   thinking_icon()  -> blue   (Whisper is transcribing)

use tray_icon::Icon;

const ICON_SIZE: u32 = 30;
const ICON_RGBA: &[u8] = include_bytes!("../assets/icon_30.rgba");

pub fn idle_icon() -> Icon {
    tinted_icon([220, 220, 220])
}

pub fn recording_icon() -> Icon {
    tinted_icon([160, 50, 50])
}

pub fn thinking_icon() -> Icon {
    tinted_icon([48, 120, 210])
}

/// Create a tinted copy of the icon. Uses the source alpha as a mask
/// and replaces the RGB channels with the given color.
fn tinted_icon(color: [u8; 3]) -> Icon {
    let mut px = Vec::with_capacity(ICON_RGBA.len());
    for chunk in ICON_RGBA.chunks_exact(4) {
        let a = chunk[3];
        px.push(color[0]);
        px.push(color[1]);
        px.push(color[2]);
        px.push(a);
    }
    Icon::from_rgba(px, ICON_SIZE, ICON_SIZE).expect("icon creation failed")
}
