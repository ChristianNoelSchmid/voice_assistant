mod audio;
mod commands;
mod config;
mod dispatcher;
mod recognizer;
mod speaker;
mod tasks;
mod tokens;

use std::sync::Arc;

use audio::AudioCapture;
use commands::{ClockCommand, DynCommandHandler, RemindCommand, ShoppingCommand, UnhandledCommand};
use config::Config;
use recognizer::SpeechRecognizer;
use speaker::piper::PiperSpeaker;
use tasks::vikunja::VikunjaClient;

use crate::dispatcher::Dispatcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load VIKUNJA_TOKEN from .env if present; it may also come from the real environment.
    dotenvy::dotenv().ok();

    let config = Config::load("config.json")?;

    let vikunja_token = std::env::var("VIKUNJA_TOKEN")
        .map_err(|_| anyhow::anyhow!("`VIKUNJA_TOKEN` is not set"))?;

    let vikunja_client = VikunjaClient::new(config.vikunja_url.clone(), vikunja_token);
    let speaker = PiperSpeaker::new(
        config.piper_bin.clone(),
        config.piper_model.clone(),
        config.piper_sample_rate,
    );

    let handlers: Vec<Arc<dyn DynCommandHandler>> = vec![
        Arc::new(RemindCommand::new(
            vikunja_client.clone(),
            speaker.clone(),
            config.vikunja_project_id,
        )),
        Arc::new(ShoppingCommand::new(
            vikunja_client,
            speaker.clone(),
            config.vikunja_shopping_project_id,
        )),
        Arc::new(ClockCommand::new(speaker.clone())),
        Arc::new(UnhandledCommand::new(speaker.clone())),
    ];
    let mut dispatcher = Dispatcher::new(handlers);

    eprintln!("Loading model from '{}'...", config.vosk_model);
    let mut recognizer = SpeechRecognizer::new(&config.vosk_model, dispatcher::WAKE_PHRASE);

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
