use crate::audio::SAMPLE_RATE;
use vosk::{DecodingState, Model, Recognizer};

/// A speech recognition result emitted by [`SpeechRecognizer::process`].
pub enum RecognitionEvent {
    /// An in-progress hypothesis — text may change as more audio arrives.
    Partial(String),
    /// A stable, end-of-utterance transcript.
    Finalized(String),
}

/// Wraps a Vosk [`Recognizer`] and converts raw i16 audio chunks into [`RecognitionEvent`]s.
pub struct SpeechRecognizer {
    // Declaration order = drop order: recognizer must be freed before the model it borrows.
    recognizer: Recognizer,
    _model: Model,
}

impl SpeechRecognizer {
    pub fn new(model_path: &str) -> Self {
        let model = Model::new(model_path).expect("Failed to load Vosk model");
        let mut recognizer =
            Recognizer::new(&model, SAMPLE_RATE as f32).expect("Failed to create recognizer");
        recognizer.set_words(false);
        Self {
            recognizer,
            _model: model,
        }
    }

    pub fn process(&mut self, samples: &[i16]) -> Option<RecognitionEvent> {
        match self.recognizer.accept_waveform(samples) {
            DecodingState::Finalized => {
                let text = self
                    .recognizer
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
                let partial = self.recognizer.partial_result().partial.to_owned();
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
