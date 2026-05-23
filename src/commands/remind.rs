use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::tokens::{PeriodToken, RemindToken, TimeToken, Token};

use super::CommandHandler;

pub struct RemindMatch {
    pub remind: RemindToken,
    pub period: Option<PeriodToken>,
    pub time: Option<TimeToken>,
}

pub struct RemindCommand;

impl CommandHandler for RemindCommand {
    type Match = RemindMatch;

    fn parse(&self, text: &str) -> Option<Self::Match> {
        let (_, remind_span) = RemindToken::parse(text)?;

        let mut consumed: Vec<Range<usize>> = vec![];

        let period = PeriodToken::parse(text).map(|(token, span)| {
            consumed.push(span);
            token
        });

        let time = TimeToken::parse(text).map(|(token, span)| {
            consumed.push(span);
            token
        });

        if period.is_none() && time.is_none() {
            return None;
        }

        let content = extract_content(text, remind_span.end, &consumed);

        Some(RemindMatch {
            remind: RemindToken { content },
            period,
            time,
        })
    }

    fn handle(&mut self, matched: Self::Match) {
        println!("[Remind] \"{}\"", matched.remind.content);
        if let Some(p) = matched.period {
            println!("  period: {:?}", p);
        }
        if let Some(t) = matched.time {
            println!("  time: {:?}", t);
        }
    }
}

/// Collects text from `from` onward, skipping the consumed spans, then
/// extracts the verb phrase following "to".
fn extract_content(text: &str, from: usize, consumed: &[Range<usize>]) -> String {
    let mut unclaimed = String::new();
    let mut pos = from;

    let mut spans: Vec<&Range<usize>> = consumed.iter().filter(|r| r.end > from).collect();
    spans.sort_by_key(|r| r.start);

    for span in &spans {
        let seg_end = span.start.max(pos);
        if pos < seg_end {
            unclaimed.push_str(&text[pos..seg_end]);
        }
        pos = span.end.max(pos);
    }
    if pos < text.len() {
        unclaimed.push_str(&text[pos..]);
    }

    // The content is the verb phrase that follows "to"
    static TO_CONTENT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\bto\s+(\S.*)").unwrap()
    });

    TO_CONTENT.captures(&unclaimed)
        .map(|caps| caps[1].split_whitespace().collect::<Vec<_>>().join(" "))
        .unwrap_or_default()
}
