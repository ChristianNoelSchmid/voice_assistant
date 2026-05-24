use async_trait::async_trait;

use crate::{commands::CommandHandler, speaker::DynSpeaker};

/// Fallback handler that prints any unrecognized command to stdout.
///
/// Always matches, so it should be last in the handler list.
pub struct UnhandledCommand {
    speaker: DynSpeaker,
}

impl UnhandledCommand {
    pub fn new(speaker: DynSpeaker) -> UnhandledCommand {
        Self { speaker }
    }
}

#[async_trait]
impl CommandHandler for UnhandledCommand {
    type Match = String;

    fn parse(&self, text: &str) -> Option<Self::Match> {
        Some(text.to_string())
    }

    async fn handle(&self, matched: Self::Match) {
        let text = format!("Quack! You said {}", matched);
        println!("{}", text);
        self.speaker.speak(text).await.unwrap();
    }
}
