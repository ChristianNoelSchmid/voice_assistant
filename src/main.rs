mod audio;
mod commands;
mod dispatcher;
mod recognizer;
mod tokens;
mod tasks;

use audio::AudioCapture;
use commands::{DynCommandHandler, PrintHandler, RemindCommand};
use dispatcher::Dispatcher;
use recognizer::SpeechRecognizer;
use tasks::vikunja::VikunjaClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let model_path = std::env::args().nth(1).unwrap_or_else(|| "model".to_string());

    let vikunja_client = VikunjaClient::from_env()?;

    eprintln!("Loading model from '{model_path}'...");
    let mut recognizer = SpeechRecognizer::new(&model_path);
    let handlers: Vec<Box<dyn DynCommandHandler>> = vec![
        Box::new(RemindCommand::new(vikunja_client)),
        Box::new(PrintHandler),
    ];
    let mut dispatcher = Dispatcher::new(handlers);
    let audio = AudioCapture::new();

    eprintln!("Ready. Say '{}' to activate.\n", dispatcher::WAKE_PHRASE);

    loop {
        let result = tokio::task::block_in_place(|| {
            let chunk = audio.rx.recv()?;
            Ok::<_, std::sync::mpsc::RecvError>(recognizer.process(&chunk))
        });
        match result {
            Ok(Some(event)) => dispatcher.dispatch(event).await,
            Ok(None) => {}
            Err(_) => break,
        }
    }
    Ok(())
}
