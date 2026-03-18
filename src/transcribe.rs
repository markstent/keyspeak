use anyhow::Result;
use std::sync::OnceLock;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

static WHISPER_CTX: OnceLock<WhisperContext> = OnceLock::new();

pub fn transcribe(samples: &[f32], model_path: &str, language: &str) -> Result<String> {
    // Too short to contain real speech
    if samples.len() < 1600 {
        return Ok(String::new());
    }

    let ctx = WHISPER_CTX.get_or_init(|| {
        WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .unwrap_or_else(|e| {
                panic!(
                    "Could not load Whisper model at '{}'.\n\
             Make sure you ran the model download command.\nError: {:?}",
                    model_path, e
                )
            })
    });

    let mut state = ctx
        .create_state()
        .map_err(|e| anyhow::anyhow!("Whisper state error: {:?}", e))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(language));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);

    state
        .full(params, samples)
        .map_err(|e| anyhow::anyhow!("Transcription failed: {:?}", e))?;

    let n = state
        .full_n_segments()
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let mut text = String::new();
    for i in 0..n {
        if let Ok(seg) = state.full_get_segment_text(i) {
            text.push_str(&seg);
        }
    }
    let cleaned = strip_noise_artifacts(text.trim());
    Ok(cleaned)
}

/// Remove non-speech artifacts that Whisper hallucinates from silence or background noise.
/// Strips bracketed/parenthesized annotations like [crickets chirping], (background noise),
/// *music playing*, and common silence hallucinations.
fn strip_noise_artifacts(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '[' => skip_until(&mut chars, ']'),
            '(' => skip_until(&mut chars, ')'),
            '*' => {
                // Only skip *starred phrases* (annotations), not single asterisks
                chars.next();
                let rest: String = chars.clone().collect();
                if let Some(end) = rest.find('*') {
                    for _ in 0..=end {
                        chars.next();
                    }
                } else {
                    result.push('*');
                }
            }
            _ => {
                result.push(ch);
                chars.next();
            }
        }
    }

    let result = result.trim().to_string();

    // Common Whisper silence hallucinations (exact matches after stripping)
    const NOISE_PHRASES: &[&str] = &[
        "thank you",
        "thanks for watching",
        "you",
        "bye",
        "goodbye",
        "the end",
        "...",
        "subtitles by the amara.org community",
    ];

    let lower = result.to_lowercase();
    let trimmed = lower.trim_matches(|c: char| c == '.' || c == '!' || c == ' ');
    if NOISE_PHRASES.contains(&trimmed) {
        return String::new();
    }

    result
}

fn skip_until(chars: &mut std::iter::Peekable<std::str::Chars>, closing: char) {
    chars.next(); // consume opening bracket
    for ch in chars.by_ref() {
        if ch == closing {
            break;
        }
    }
}
