//! Waveform menu bar icons drawn entirely in code — no image files needed.
//!
//! The icon shows 5 vertical bars of varying heights forming an audio
//! waveform / equaliser shape. Three colour states:
//!   idle_icon()      → grey   (app is running, not active)
//!   recording_icon() → red    (microphone is open, recording)
//!   thinking_icon()  → blue   (Whisper is transcribing)

use tray_icon::Icon;

/// Heights of each bar as a fraction of the full icon height.
/// Chosen to form an asymmetric waveform that looks natural.
const BAR_HEIGHTS: [f32; 5] = [0.35, 0.78, 0.52, 0.88, 0.42];

const BAR_WIDTH: u32 = 3; // pixels wide per bar
const BAR_GAP: u32 = 2; // pixels between bars
const ICON_SIZE: u32 = 22; // total icon dimension (22×22 px)

pub fn idle_icon() -> Icon {
    waveform([125, 125, 125, 255])
}
pub fn recording_icon() -> Icon {
    waveform([210, 48, 48, 255])
}
pub fn thinking_icon() -> Icon {
    waveform([48, 120, 210, 255])
}

fn waveform(color: [u8; 4]) -> Icon {
    let size = ICON_SIZE;
    let mut px = vec![0u8; (size * size * 4) as usize];

    let num_bars = BAR_HEIGHTS.len() as u32;
    // Total pixel width of all bars and gaps combined
    let total_w = num_bars * BAR_WIDTH + (num_bars - 1) * BAR_GAP;
    // Left offset to centre the waveform in the icon
    let origin_x = (size.saturating_sub(total_w)) / 2;

    for (i, &frac) in BAR_HEIGHTS.iter().enumerate() {
        let bar_h = ((size as f32 * frac) as u32).clamp(2, size);
        let bar_x = origin_x + i as u32 * (BAR_WIDTH + BAR_GAP);
        // Vertically centre each bar
        let bar_y = (size - bar_h) / 2;

        for dy in 0..bar_h {
            for dx in 0..BAR_WIDTH {
                let x = bar_x + dx;
                let y = bar_y + dy;
                if x >= size || y >= size {
                    continue;
                }

                // Soften the top and bottom edge of each bar
                let alpha_frac = if dy == 0 || dy == bar_h - 1 {
                    0.55_f32
                } else {
                    1.0_f32
                };

                let idx = ((y * size + x) * 4) as usize;
                px[idx] = color[0];
                px[idx + 1] = color[1];
                px[idx + 2] = color[2];
                px[idx + 3] = (color[3] as f32 * alpha_frac) as u8;
            }
        }
    }

    Icon::from_rgba(px, size, size).expect("waveform icon creation failed")
}
