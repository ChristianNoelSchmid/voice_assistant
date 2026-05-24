mod normalize;
mod period;
mod remind;
mod time;

pub use normalize::normalize;
#[allow(unused_imports)]
pub use period::{PeriodSpec, PeriodToken, Recurrence, Weekday};
pub use remind::RemindToken;
pub use time::TimeToken;

/// A value parseable from a normalized transcript string.
///
/// Returns the parsed token and the byte range it occupied in the input,
/// so callers can subtract consumed spans when extracting remaining content.
pub trait Token: Sized {
    fn parse(text: &str) -> Option<(Self, std::ops::Range<usize>)>;
}
