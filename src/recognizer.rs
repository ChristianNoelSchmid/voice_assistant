use crate::audio::SAMPLE_RATE;
use vosk::{DecodingState, Model, Recognizer};

/// A speech recognition result emitted by [`SpeechRecognizer::process`].
pub enum RecognitionEvent {
    /// An in-progress hypothesis — text may change as more audio arrives.
    Partial(String),
    /// A stable, end-of-utterance transcript.
    Finalized(String),
}

/// Controls which internal Vosk recognizer [`SpeechRecognizer`] routes audio to.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RecognizerMode {
    /// Vocabulary is constrained to the wake phrase for better noise rejection.
    Idle,
    /// Full unconstrained recognition for command transcription.
    Active,
}

/// Wraps two Vosk recognizers — one grammar-constrained for wake-word detection, one
/// unrestricted for command transcription — and converts raw i16 audio chunks into
/// [`RecognitionEvent`]s.
///
/// Call [`set_mode`] whenever the dispatcher transitions state so the correct
/// recognizer is used.  The incoming recognizer is reset on each switch to discard
/// any partial hypothesis left over from the previous utterance.
///
/// [`set_mode`]: SpeechRecognizer::set_mode
pub struct SpeechRecognizer {
    // Declaration order = drop order: recognizers must be freed before the model they borrow.
    /// Grammar-constrained to `[wake_phrase, "[unk]"]`; active while idle.
    idle: Recognizer,
    /// Unrestricted recognizer; active after the wake phrase fires.
    active: Recognizer,
    mode: RecognizerMode,
    _model: Model,
}

impl SpeechRecognizer {
    /// Creates both recognizers from `model_path`.
    ///
    /// `wake_phrase` is the word used for grammar-constrained idle recognition.
    /// If the loaded model does not support grammar constraints (precompiled HCLG
    /// graphs), the idle recognizer falls back to full recognition with a warning.
    pub fn new(model_path: &str, wake_phrase: &str) -> Self {
        let model = Model::new(model_path).expect("Failed to load Vosk model");

        let mut idle = Recognizer::new_with_grammar(
            &model,
            SAMPLE_RATE as f32,
            &[wake_phrase, "[unk]"],
        )
        .unwrap_or_else(|| {
            eprintln!(
                "[Warning] Model does not support grammar constraints — \
                 idle recognizer will use full vocabulary"
            );
            Recognizer::new(&model, SAMPLE_RATE as f32).expect("Failed to create idle recognizer")
        });
        idle.set_words(false);

        let mut active =
            Recognizer::new(&model, SAMPLE_RATE as f32).expect("Failed to create command recognizer");
        active.set_words(false);

        Self {
            idle,
            active,
            mode: RecognizerMode::Idle,
            _model: model,
        }
    }

    /// Switches the active recognizer and resets it to discard stale partial state.
    /// No-ops when the mode is unchanged.
    pub fn set_mode(&mut self, mode: RecognizerMode) {
        if mode == self.mode {
            return;
        }
        self.mode = mode;
        match self.mode {
            RecognizerMode::Idle => self.idle.reset(),
            RecognizerMode::Active => self.active.reset(),
        }
    }

    /// Feeds a PCM chunk to the active recognizer and returns a recognition event if one is ready.
    pub fn process(&mut self, samples: &[i16]) -> Option<RecognitionEvent> {
        let rec = match self.mode {
            RecognizerMode::Idle => &mut self.idle,
            RecognizerMode::Active => &mut self.active,
        };
        let state = rec.accept_waveform(samples);
        match state {
            DecodingState::Finalized => {
                let text = rec
                    .result()
                    .single()
                    .map(|r| r.text.to_owned())
                    .unwrap_or_default();
                if text.is_empty() {
                    None
                } else {
                    Some(RecognitionEvent::Finalized(text))
                }
            }
            DecodingState::Running => {
                let partial = rec.partial_result().partial.to_owned();
                if partial.is_empty() {
                    None
                } else {
                    Some(RecognitionEvent::Partial(partial))
                }
            }
            DecodingState::Failed => {
                eprintln!("Decoding error");
                None
            }
        }
    }
}
