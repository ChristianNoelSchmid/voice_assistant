use regex::Regex;
use std::sync::LazyLock;

use super::Token;

pub struct RemindToken;

static REMIND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\breminds?\b").unwrap()
});

impl Token for RemindToken {
    fn parse(text: &str) -> Option<Self> {
        REMIND.is_match(text).then_some(RemindToken)
    }
}
