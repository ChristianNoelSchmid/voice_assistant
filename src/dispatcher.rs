use crate::commands::DynCommandHandler;
use crate::recognizer::RecognitionEvent;
use std::time::{Duration, Instant};

pub const WAKE_PHRASE: &str = "popcorn";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
enum State {
    /// Waiting for the wake phrase.
    Idle,
    /// Wake phrase heard; accepting commands until `deadline`.
    Active { deadline: Instant },
}

/// Routes [`RecognitionEvent`]s to command handlers after the wake phrase is detected.
pub struct Dispatcher {
    handlers: Vec<Box<dyn DynCommandHandler>>,
    state: State,
}

impl Dispatcher {
    pub fn new(handlers: Vec<Box<dyn DynCommandHandler>>) -> Self {
        Self {
            handlers,
            state: State::Idle,
        }
    }

    pub async fn dispatch(&mut self, event: RecognitionEvent) {
        match event {
            RecognitionEvent::Partial(text) => self.process(&text, false).await,
            RecognitionEvent::Finalized(text) => self.process(&text, true).await,
        }
        self.check_timeout();
    }

    async fn process(&mut self, text: &str, is_final: bool) {
        let lower = text.to_lowercase();
        match self.state {
            // Wake phrase is checked on partial results so activation happens before Vosk finalizes.
            State::Idle => {
                if lower.contains(WAKE_PHRASE) {
                    eprintln!(
                        "[Activated] Listening for a command ({} s timeout)...",
                        COMMAND_TIMEOUT.as_secs()
                    );
                    self.state = State::Active {
                        deadline: Instant::now() + COMMAND_TIMEOUT,
                    };
                }
            }
            State::Active { ref mut deadline } => {
                if is_final {
                    for handler in &mut self.handlers {
                        if handler.try_handle(text).await {
                            break;
                        }
                    }
                    // Reset deadline after each command so the user can chain commands without re-waking.
                    *deadline = Instant::now() + COMMAND_TIMEOUT;
                }
            }
        }
    }

    fn check_timeout(&mut self) {
        if let State::Active { deadline } = self.state {
            if Instant::now() >= deadline {
                eprintln!("[Timed out — back to idle]");
                self.state = State::Idle;
            }
        }
    }
}
