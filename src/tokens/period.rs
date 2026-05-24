use regex::Regex;
use std::sync::LazyLock;

use super::Token;

/// A day of the week parsed from text (e.g. "monday", "saturday").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// How often a scheduled task should repeat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recurrence {
    /// Occurs a single time with no repeat.
    Once,
    Daily,
    Weekly,
    Monthly,
}

/// Which day a period token refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodSpec {
    /// A named weekday (e.g. "on Friday").
    Weekday(Weekday),
    /// A day-of-month ordinal 1–31 (e.g. "on the 3rd of the month").
    MonthDay(u8),
    /// Every calendar day (e.g. "every day").
    Daily,
}

/// Represents a scheduling period extracted from an utterance,
/// combining *which day* with *how often*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodToken {
    pub spec: PeriodSpec,
    pub recurrence: Recurrence,
}

// "every day"
static EVERY_DAY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bevery\s+day\b").unwrap());

// "every Saturday", "every Monday"
static EVERY_WEEKDAY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bevery\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b")
        .unwrap()
});

// "on Sunday" (once) or "on Sundays" (weekly — plural 's' captured in group 2)
static ON_WEEKDAY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bon\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)(s)?\b")
        .unwrap()
});

// "every 1st of the month" / "on the 3rd of the month"
static MONTH_DAY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:\bevery|\bon\s+the)\s+(\d+)(?:st|nd|rd|th)\s+of\s+the\s+month").unwrap()
});

fn weekday_from_str(s: &str) -> Option<Weekday> {
    match s.to_lowercase().as_str() {
        "monday" => Some(Weekday::Monday),
        "tuesday" => Some(Weekday::Tuesday),
        "wednesday" => Some(Weekday::Wednesday),
        "thursday" => Some(Weekday::Thursday),
        "friday" => Some(Weekday::Friday),
        "saturday" => Some(Weekday::Saturday),
        "sunday" => Some(Weekday::Sunday),
        _ => None,
    }
}

impl Token for PeriodToken {
    fn parse(text: &str) -> Option<(Self, std::ops::Range<usize>)> {
        // "every day" is checked before "every [weekday]" to avoid a false
        // match on a hypothetical weekday named "day".
        if let Some(m) = EVERY_DAY.find(text) {
            return Some((
                PeriodToken {
                    spec: PeriodSpec::Daily,
                    recurrence: Recurrence::Daily,
                },
                m.range(),
            ));
        }

        // Month-day patterns are checked next: they contain "of the month"
        // so they won't collide with weekday patterns.
        if let Some(caps) = MONTH_DAY.captures(text) {
            let day: u8 = caps[1].parse().ok()?;
            return Some((
                PeriodToken {
                    spec: PeriodSpec::MonthDay(day.clamp(1, 31)),
                    recurrence: Recurrence::Monthly,
                },
                caps.get(0).unwrap().range(),
            ));
        }

        if let Some(caps) = EVERY_WEEKDAY.captures(text) {
            return Some((
                PeriodToken {
                    spec: PeriodSpec::Weekday(weekday_from_str(&caps[1])?),
                    recurrence: Recurrence::Weekly,
                },
                caps.get(0).unwrap().range(),
            ));
        }

        if let Some(caps) = ON_WEEKDAY.captures(text) {
            let recurrence = if caps.get(2).is_some() {
                Recurrence::Weekly // "on Sundays"
            } else {
                Recurrence::Once // "on Sunday"
            };
            return Some((
                PeriodToken {
                    spec: PeriodSpec::Weekday(weekday_from_str(&caps[1])?),
                    recurrence,
                },
                caps.get(0).unwrap().range(),
            ));
        }

        None
    }
}
