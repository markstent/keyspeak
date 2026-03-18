use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use crossbeam_channel::Sender;
use rubato::{FftFixedInOut, Resampler};

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
pub fn resample_to_16k(samples: Vec<f32>, from_rate: u32) -> Vec<f32> {
    if from_rate == WHISPER_SAMPLE_RATE || samples.is_empty() {
        return samples;
    }

    let chunk = 1024.min(samples.len());
    let mut r =
        match FftFixedInOut::<f32>::new(from_rate as usize, WHISPER_SAMPLE_RATE as usize, chunk, 1)
        {
            Ok(r) => r,
            Err(e) => {
                log::error!("Resampler init: {}", e);
                return samples;
            }
        };

    let ratio = WHISPER_SAMPLE_RATE as f64 / from_rate as f64;
    let mut out = Vec::with_capacity((samples.len() as f64 * ratio) as usize + 64);
    let mut pos = 0;

    while pos + chunk <= samples.len() {
        if let Ok(res) = r.process(&[&samples[pos..pos + chunk]], None) {
            out.extend_from_slice(&res[0]);
        }
        pos += chunk;
    }

    // Handle the remaining samples (pad to fill last chunk)
    if pos < samples.len() {
        let real = samples.len() - pos;
        let mut tail = samples[pos..].to_vec();
        tail.resize(chunk, 0.0);
        if let Ok(res) = r.process(&[&tail], None) {
            let take = (real as f64 * ratio) as usize;
            out.extend_from_slice(&res[0][..take.min(res[0].len())]);
        }
    }
    out
}
