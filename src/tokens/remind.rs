use regex::Regex;
use std::sync::LazyLock;

use super::Token;

pub struct RemindToken {
    /// Content extracted by RemindCommand after all other token spans are subtracted.
    /// Empty when returned from Token::parse — the command fills this in.
    pub content: String,
}

static REMIND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\breminds?\b").unwrap()
});

impl Token for RemindToken {
    fn parse(text: &str) -> Option<(Self, std::ops::Range<usize>)> {
        let m = REMIND.find(text)?;
        Some((RemindToken { content: String::new() }, m.range()))
    }
}
