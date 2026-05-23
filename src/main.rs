mod audio;
mod commands;
mod dispatcher;
mod recognizer;
mod tokens;

use audio::AudioCapture;
use commands::{DynCommandHandler, PrintHandler, RemindCommand};
use dispatcher::Dispatcher;
use recognizer::SpeechRecognizer;

fn main() {
    let model_path = std::env::args().nth(1).unwrap_or_else(|| "model".to_string());

    eprintln!("Loading model from '{model_path}'...");
    let mut recognizer = SpeechRecognizer::new(&model_path);
    let handlers: Vec<Box<dyn DynCommandHandler>> = vec![
        Box::new(RemindCommand),
        Box::new(PrintHandler),
    ];
    let mut dispatcher = Dispatcher::new(handlers);
    let audio = AudioCapture::new();

    eprintln!("Ready. Say '{}' to activate.\n", dispatcher::WAKE_PHRASE);

    for chunk in &audio.rx {
        if let Some(event) = recognizer.process(&chunk) {
            dispatcher.dispatch(event);
        }
    }
}
