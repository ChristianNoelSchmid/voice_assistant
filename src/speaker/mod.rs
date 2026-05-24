pub mod piper;

use std::sync::Arc;

use async_trait::async_trait;

/// Errors that can occur when synthesising or playing back speech.
#[derive(Debug, thiserror::Error)]
pub enum SpeakerError {
    /// An I/O error occurred while spawning the TTS process or reading/writing its streams.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The TTS process exited with a non-zero status.
    #[error("TTS process failed (exit code {code:?}): {stderr}")]
    ProcessFailed { code: Option<i32>, stderr: String },

    /// An error occurred during audio playback.
    #[error("audio playback error: {0}")]
    Audio(String),

}

/// Abstraction over a text-to-speech backend.
#[async_trait]
pub trait Speaker {
    /// Synthesise `text` and play it back through the default audio output device.
    ///
    /// Awaiting this future blocks until playback has fully completed.
    async fn speak(&self, text: String) -> Result<(), SpeakerError>;
}

/// A reference-counted [`dyn Speaker`] ready to be shared across tasks.
pub type DynSpeaker = Arc<dyn Speaker + Send + Sync>;
