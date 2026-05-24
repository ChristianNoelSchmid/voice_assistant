mod audio;
mod commands;
mod dispatcher;
mod recognizer;
mod speaker;
mod tasks;
mod tokens;

use std::sync::Arc;

use audio::AudioCapture;
use commands::{ClockCommand, DynCommandHandler, RemindCommand, ShoppingCommand, UnhandledCommand};
use recognizer::SpeechRecognizer;
use speaker::piper::PiperSpeaker;
use tasks::vikunja::VikunjaClient;

use crate::dispatcher::Dispatcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let model_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "lib/vosk_model".to_string());

    let remind_project_id = std::env::var("VIKUNJA_PROJECT_ID")
        .map_err(|_| anyhow::anyhow!("`VIKUNJA_PROJECT_ID` is not set"))?
        .parse::<u64>()
        .map_err(|e| anyhow::anyhow!("`VIKUNJA_PROJECT_ID` is not a valid integer: {e}"))?;
    let shopping_project_id = std::env::var("VIKUNJA_SHOPPING_PROJECT_ID")
        .map_err(|_| anyhow::anyhow!("`VIKUNJA_SHOPPING_PROJECT_ID` is not set"))?
        .parse::<u64>()
        .map_err(|e| {
            anyhow::anyhow!("`VIKUNJA_SHOPPING_PROJECT_ID` is not a valid integer: {e}")
        })?;

    let vikunja_client = VikunjaClient::from_env()?;
    let speaker = PiperSpeaker::from_env()?;

    let handlers: Vec<Arc<dyn DynCommandHandler>> = vec![
        Arc::new(RemindCommand::new(
            vikunja_client.clone(),
            speaker.clone(),
            remind_project_id,
        )),
        Arc::new(ShoppingCommand::new(
            vikunja_client,
            speaker.clone(),
            shopping_project_id,
        )),
        Arc::new(ClockCommand::new(speaker.clone())),
        Arc::new(UnhandledCommand::new(speaker.clone())),
    ];
    let mut dispatcher = Dispatcher::new(handlers);

    eprintln!("Loading model from '{model_path}'...");
    let mut recognizer = SpeechRecognizer::new(&model_path, dispatcher::WAKE_PHRASE);

    let audio = AudioCapture::new();

    eprintln!("Ready. Say '{}' to activate.\n", dispatcher::WAKE_PHRASE);
    speaker
        .speak(format!(
            "Ready. Say '{}' to activate.",
            dispatcher::WAKE_PHRASE
        ))
        .await?;

    loop {
        // block_in_place keeps the blocking recv + Vosk decoding off the async executor thread.
        let result = tokio::task::block_in_place(|| {
            let chunk = audio.rx.recv()?;
            Ok::<_, std::sync::mpsc::RecvError>(recognizer.process(&chunk))
        });
        match result {
            Ok(Some(event)) => {
                let mode = dispatcher.dispatch(event).await;
                recognizer.set_mode(mode);
            }
            Ok(None) => {}
            Err(_) => break, // audio channel closed — microphone disconnected or stream error
        }
    }
    Ok(())
}
