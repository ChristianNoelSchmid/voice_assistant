use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::speaker::DynSpeaker;

use super::{Speaker, SpeakerError};

/// [`Speaker`] backed by a local [Piper](https://github.com/rhasspy/piper) process.
///
/// Requires the `piper` binary to be on `PATH`. Text is written to piper's
/// stdin; raw 16-bit PCM is read from its stdout and played back via cpal.
pub struct PiperSpeaker {
    bin_path: String,
    model_path: String,
    sample_rate: u32,
}

impl PiperSpeaker {
    /// Construct a [`PiperSpeaker`] from config values.
    ///
    /// Paths are validated by [`Config::validate`] before this is called, so
    /// this constructor is infallible.
    pub fn new(bin_path: String, model_path: String, sample_rate: u32) -> DynSpeaker {
        Arc::new(Self {
            bin_path,
            model_path,
            sample_rate,
        })
    }
}

#[async_trait]
impl Speaker for PiperSpeaker {
    async fn speak(&self, text: String) -> Result<(), SpeakerError> {
        let mut child = Command::new(&self.bin_path)
            .args(["--model", &self.model_path, "--output-raw"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Write the text and close stdin so piper knows input is done.
        let mut stdin = child.stdin.take().expect("stdin was piped");
        stdin.write_all(text.as_bytes()).await?;
        drop(stdin);

        let output = child.wait_with_output().await?;
        if !output.status.success() {
            return Err(SpeakerError::ProcessFailed {
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        // Piper --output-raw emits 16-bit signed little-endian mono PCM.
        let samples: Vec<i16> = output
            .stdout
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();

        let sample_rate = self.sample_rate;
        tokio::task::spawn_blocking(move || play_samples(samples, sample_rate))
            .await
            .map_err(|e| SpeakerError::Audio(e.to_string()))??;

        Ok(())
    }
}

/// Play a buffer of mono 16-bit PCM samples through the default output device.
///
/// Blocks the calling thread until all samples have been drained by cpal.
fn play_samples(samples: Vec<i16>, sample_rate: u32) -> Result<(), SpeakerError> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| SpeakerError::Audio("no default output device".into()))?;

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let samples = Arc::new(samples);
    let pos = Arc::new(AtomicUsize::new(0));

    let (done_tx, done_rx) = tokio::sync::oneshot::channel::<()>();

    let cb_samples = Arc::clone(&samples);
    let cb_pos = Arc::clone(&pos);
    // Option so the FnMut closure can consume the sender exactly once.
    let mut done_tx = Some(done_tx);

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let p = cb_pos.load(Ordering::Relaxed);
                let remaining = cb_samples.len().saturating_sub(p);
                let to_copy = data.len().min(remaining);
                data[..to_copy].copy_from_slice(&cb_samples[p..p + to_copy]);
                // Zero-fill any trailing frames after the sample buffer is exhausted.
                data[to_copy..].fill(0);
                cb_pos.fetch_add(to_copy, Ordering::Relaxed);
                if to_copy < data.len() {
                    if let Some(tx) = done_tx.take() {
                        let _ = tx.send(());
                    }
                }
            },
            |err| eprintln!("audio output error: {err}"),
            None,
        )
        .map_err(|e| SpeakerError::Audio(e.to_string()))?;

    stream
        .play()
        .map_err(|e| SpeakerError::Audio(e.to_string()))?;

    // Block until the callback signals that all samples have been written.
    done_rx.blocking_recv().ok();

    Ok(())
}
