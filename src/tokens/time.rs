use regex::Regex;
use std::sync::LazyLock;

use super::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeToken {
    pub hour: u8,   // 0–23
    pub minute: u8, // 0–59
}

// "at 7 AM", "at 9:30 PM"
static AT_TIME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bat\s+(\d{1,2})(?::(\d{2}))?\s*(am|pm)\b").unwrap()
});

static AT_NOON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bat\s+noon\b").unwrap()
});

static AT_MIDNIGHT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bat\s+midnight\b").unwrap()
});

impl Token for TimeToken {
    fn parse(text: &str) -> Option<Self> {
        if AT_NOON.is_match(text) {
            return Some(TimeToken { hour: 12, minute: 0 });
        }

        if AT_MIDNIGHT.is_match(text) {
            return Some(TimeToken { hour: 0, minute: 0 });
        }

        if let Some(caps) = AT_TIME.captures(text) {
            let mut hour: u8 = caps[1].parse().ok()?;
            let minute: u8 = caps.get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);

            match caps[3].to_lowercase().as_str() {
                "pm" if hour != 12 => hour = hour.saturating_add(12),
                "am" if hour == 12 => hour = 0,
                _ => {}
            }

            return Some(TimeToken { hour, minute });
        }

        None
    }
}
