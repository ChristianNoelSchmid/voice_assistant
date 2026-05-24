use std::ops::Range;
use std::sync::LazyLock;

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use regex::Regex;

use async_trait::async_trait;

use crate::tokens::{PeriodSpec, PeriodToken, Recurrence, RemindToken, TimeToken, Token, Weekday};
use crate::tasks::DynTaskClient;

use super::CommandHandler;

pub struct RemindMatch {
    pub remind: RemindToken,
    pub period: Option<PeriodToken>,
    pub time: Option<TimeToken>,
}

pub struct RemindCommand {
    client: DynTaskClient,
}

impl RemindCommand {
    pub fn new(client: DynTaskClient) -> Self {
        Self { client }
    }
}

#[async_trait]
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

    async fn handle(&mut self, matched: Self::Match) {
        let title = &matched.remind.content;
        println!("[Remind] \"{}\"", title);
        if let Some(p) = matched.period {
            println!("  period: {:?}", p);
        }
        if let Some(t) = matched.time {
            println!("  time: {:?}", t);
        }

        let due_date = compute_due_date(matched.period.as_ref(), matched.time.as_ref());
        let (repeat_after, repeat_mode) = compute_repeat(matched.period.as_ref());

        match self.client.create_task(title, due_date, repeat_after, repeat_mode).await {
            Ok(()) => println!("[Remind] Task created in Vikunja."),
            Err(e) => eprintln!("[Remind] Failed to create task: {e}"),
        }
    }
}

fn compute_due_date(period: Option<&PeriodToken>, time: Option<&TimeToken>) -> Option<DateTime<Utc>> {
    let today = Local::now().date_naive();

    let naive_time = time
        .and_then(|t| NaiveTime::from_hms_opt(t.hour as u32, t.minute as u32, 0))
        .unwrap_or_else(|| NaiveTime::from_hms_opt(12, 0, 0).unwrap());

    let date = match period {
        Some(p) => match p.spec {
            PeriodSpec::Daily => today,
            PeriodSpec::Weekday(wd) => next_weekday(today, wd),
            PeriodSpec::MonthDay(day) => next_month_day(today, day),
        },
        None => today,
    };

    Local
        .from_local_datetime(&date.and_time(naive_time))
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

fn compute_repeat(period: Option<&PeriodToken>) -> (Option<i64>, Option<i32>) {
    match period {
        None => (None, None),
        Some(p) => match p.recurrence {
            Recurrence::Once => (None, None),
            Recurrence::Daily => (Some(86400), None),
            Recurrence::Weekly => (Some(604800), None),
            // repeat_mode=1: Vikunja repeats on the same day-of-month; repeat_after=1 means every 1 month
            Recurrence::Monthly => (Some(1), Some(1)),
        },
    }
}

fn next_weekday(from: NaiveDate, wd: Weekday) -> NaiveDate {
    use chrono::Weekday as CWd;
    let target = match wd {
        Weekday::Monday => CWd::Mon,
        Weekday::Tuesday => CWd::Tue,
        Weekday::Wednesday => CWd::Wed,
        Weekday::Thursday => CWd::Thu,
        Weekday::Friday => CWd::Fri,
        Weekday::Saturday => CWd::Sat,
        Weekday::Sunday => CWd::Sun,
    };
    let delta = (target.num_days_from_monday() + 7 - from.weekday().num_days_from_monday()) % 7;
    from + chrono::Duration::days(if delta == 0 { 7 } else { delta as i64 })
}

fn next_month_day(from: NaiveDate, day: u8) -> NaiveDate {
    let (year, month) = (from.year(), from.month());
    if let Some(d) = NaiveDate::from_ymd_opt(year, month, day as u32) {
        if d >= from {
            return d;
        }
    }
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let last_day = NaiveDate::from_ymd_opt(if nm == 12 { ny + 1 } else { ny }, if nm == 12 { 1 } else { nm + 1 }, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day();
    NaiveDate::from_ymd_opt(ny, nm, (day as u32).min(last_day)).unwrap()
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
