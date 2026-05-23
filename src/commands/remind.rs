use crate::tokens::{PeriodToken, RemindToken, TimeToken, Token};

use super::CommandHandler;

pub struct RemindMatch {
    pub period: Option<PeriodToken>,
    pub time: Option<TimeToken>,
}

pub struct RemindCommand;

impl CommandHandler for RemindCommand {
    type Match = RemindMatch;

    fn parse(&self, text: &str) -> Option<Self::Match> {
        RemindToken::parse(text)?;
        let period = PeriodToken::parse(text);
        let time = TimeToken::parse(text);
        if period.is_none() && time.is_none() {
            return None;
        }
        Some(RemindMatch { period, time })
    }

    fn handle(&mut self, matched: Self::Match) {
        println!("[Remind]");
        if let Some(p) = matched.period {
            println!("  period: {:?}", p);
        }
        if let Some(t) = matched.time {
            println!("  time:   {:?}", t);
        }
    }
}
