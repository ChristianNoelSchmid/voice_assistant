use crate::commands::DynCommandHandler;
use crate::recognizer::{RecognitionEvent, RecognizerMode};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub const WAKE_PHRASE: &str = "harlequin";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
enum Behavior {
    /// Waiting for the wake phrase.
    Idle,
    /// Wake phrase heard; accepting commands until `deadline`.
    Active { deadline: Instant },
}

/// Routes [`RecognitionEvent`]s to command handlers after the wake phrase is detected.
pub struct Dispatcher {
    handlers: Vec<Arc<dyn DynCommandHandler>>,
    behavior: Behavior,
}

impl Dispatcher {
    pub fn new(handlers: Vec<Arc<dyn DynCommandHandler>>) -> Self {
        Self {
            handlers,
            behavior: Behavior::Idle,
        }
    }

    fn check_timeout(&mut self) {
        if let Behavior::Active { deadline } = self.behavior {
            if Instant::now() >= deadline {
                eprintln!("[Timed out — back to idle]");
                self.behavior = Behavior::Idle;
            }
        }
    }

    /// Processes a recognition event and returns the [`RecognizerMode`] the caller
    /// should switch to.  The caller is responsible for forwarding this to
    /// [`SpeechRecognizer::set_mode`] so the correct Vosk recognizer is used for
    /// subsequent audio chunks.
    pub async fn dispatch(&mut self, event: RecognitionEvent) -> RecognizerMode {
        match event {
            RecognitionEvent::Partial(text) => self.process(&text, false).await,
            RecognitionEvent::Finalized(text) => self.process(&text, true).await,
        }
        self.check_timeout();
        match self.behavior {
            Behavior::Idle => RecognizerMode::Idle,
            Behavior::Active { .. } => RecognizerMode::Active,
        }
    }

    async fn process(&mut self, text: &str, is_final: bool) {
        let lower = text.to_lowercase();

        match self.behavior {
            // Wake phrase is checked on partial results so activation happens before Vosk finalizes.
            Behavior::Idle => {
                if lower.contains(WAKE_PHRASE) {
                    eprintln!(
                        "[Activated] Listening for a command ({} s timeout)...",
                        COMMAND_TIMEOUT.as_secs()
                    );
                    self.behavior = Behavior::Active {
                        deadline: Instant::now() + COMMAND_TIMEOUT,
                    };
                }
            }
            Behavior::Active { .. } => {
                if is_final {
                    let handlers = self.handlers.clone();
                    // Return to Idle before running handlers so that audio buffered
                    // while the speaker plays back cannot re-trigger commands.
                    self.behavior = Behavior::Idle;

                    for handler in handlers {
                        if handler.try_handle(text).await {
                            break;
                        }
                    }
                }
            }
        }
    }
}
