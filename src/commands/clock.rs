use chrono::{Local, Timelike};

use async_trait::async_trait;

use crate::speaker::DynSpeaker;

use super::CommandHandler;

/// Handles "remind me to X [period] [time]" utterances by creating a task in Vikunja.
pub struct ClockCommand {
    speaker: DynSpeaker,
}

impl ClockCommand {
    pub fn new(speaker: DynSpeaker) -> Self {
        Self { speaker }
    }
}

#[async_trait]
impl CommandHandler for ClockCommand {
    type Match = ();

    fn parse(&self, text: &str) -> Option<Self::Match> {
        if text.contains("time") {
            Some(())
        } else {
            None
        }
    }

    async fn handle(&self, _: Self::Match) {
        let time = Local::now();
        let (is_pm, hour) = time.hour12();
        let text = format!(
            "It is {} {} {}",
            hour,
            time.minute(),
            if is_pm { "PM" } else { "AM" }
        );
        self.speaker.speak(text).await.unwrap();
    }
}
