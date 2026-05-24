use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::sync::mpsc::{self, Receiver, SyncSender};

pub const SAMPLE_RATE: u32 = 16000;

/// Captures mono 16 kHz audio from the default input device and streams
/// i16 sample chunks over a sync channel.
pub struct AudioCapture {
    // Kept alive to hold the stream open; dropped when AudioCapture is dropped.
    _stream: Stream,
    pub rx: Receiver<Vec<i16>>,
}

impl AudioCapture {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("No input device available");

        eprintln!("Input device: {}", device.name().unwrap_or_default());

        let config = StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        // Bound of 64 caps memory if the recognizer falls behind; try_send drops rather than blocks.
        let (tx, rx): (SyncSender<Vec<i16>>, _) = mpsc::sync_channel(64);

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _| {
                    // cpal delivers f32 samples; Vosk expects i16, so convert with full-range scaling.
                    let samples: Vec<i16> = data
                        .iter()
                        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                        .collect();
                    let _ = tx.try_send(samples);
                },
                |err| eprintln!("Audio error: {err}"),
                None,
            )
            .expect("Failed to build audio stream — ensure your device supports 16 kHz mono");

        stream.play().expect("Failed to start audio stream");

        Self {
            _stream: stream,
            rx,
        }
    }
}
