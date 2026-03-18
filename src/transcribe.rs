use anyhow::Result;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub fn transcribe(samples: &[f32], model_path: &str, language: &str) -> Result<String> {
    // Too short to contain real speech
    if samples.len() < 1600 {
        return Ok(String::new());
    }

    let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| {
            anyhow::anyhow!(
                "Could not load Whisper model at '{}'.\n\
         Make sure you ran the model download command.\nError: {:?}",
                model_path,
                e
            )
        })?;

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
    Ok(text.trim().to_string())
}
