use regex::Regex;
use std::sync::LazyLock;

use super::Token;

/// Marks an utterance as a reminder request by matching the word "remind" or "reminds".
///
/// The `content` field is empty at parse time; [`RemindCommand`] fills it in after
/// subtracting all other token spans from the transcript.
pub struct RemindToken {
    pub content: String,
}

static REMIND: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\breminds?\b").unwrap());

impl Token for RemindToken {
    fn parse(text: &str) -> Option<(Self, std::ops::Range<usize>)> {
        let m = REMIND.find(text)?;
        Some((
            RemindToken {
                content: String::new(),
            },
            m.range(),
        ))
    }
}
