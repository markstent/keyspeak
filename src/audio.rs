use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use crossbeam_channel::Sender;

pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

/// Keeping this alive keeps the microphone open.
/// Drop it to stop recording.
pub struct RecordingHandle {
    _stream: cpal::Stream,
    pub device_sample_rate: u32,
}

// SAFETY: cpal::Stream is Send on macOS (CoreAudio) but the trait bound is missing.
// We only use RecordingHandle from the main thread and behind a Mutex.
unsafe impl Send for RecordingHandle {}
unsafe impl Sync for RecordingHandle {}

pub fn start_recording(sender: Sender<Vec<f32>>) -> Result<RecordingHandle> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or_else(|| {
        anyhow!(
            "No microphone found.\n\
         Fix: System Settings → Privacy & Security → Microphone → ✅ KeySpeak"
        )
    })?;

    let config = device.default_input_config()?;
    let rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let stream_config = cpal::StreamConfig::from(config.clone());

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &stream_config, channels, sender)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &stream_config, channels, sender)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &stream_config, channels, sender)?,
        fmt => return Err(anyhow!("Unsupported audio format: {:?}", fmt)),
    };

    stream.play()?;
    Ok(RecordingHandle {
        _stream: stream,
        device_sample_rate: rate,
    })
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    sender: Sender<Vec<f32>>,
) -> Result<cpal::Stream>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    Ok(device.build_input_stream(
        config,
        move |data: &[T], _| {
            // Mix down to mono f32
            let mono: Vec<f32> = data
                .chunks(channels)
                .map(|frame| {
                    frame.iter().map(|&s| f32::from_sample(s)).sum::<f32>() / channels as f32
                })
                .collect();
            let _ = sender.send(mono);
        },
        |e| log::error!("Audio stream error: {}", e),
        None,
    )?)
}

/// Resample audio from the device rate to 16 kHz (required by Whisper).
/// Uses linear interpolation - sufficient quality for speech-to-text.
pub fn resample_to_16k(samples: Vec<f32>, from_rate: u32) -> Vec<f32> {
    if from_rate == WHISPER_SAMPLE_RATE || samples.is_empty() {
        return samples;
    }

    let ratio = from_rate as f64 / WHISPER_SAMPLE_RATE as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac) + samples[idx + 1] * frac
        } else {
            samples[idx.min(samples.len() - 1)]
        };
        out.push(sample);
    }

    out
}
