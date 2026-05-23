mod normalize;
mod period;
mod remind;
mod time;

pub use normalize::normalize;
#[allow(unused_imports)]
pub use period::{PeriodSpec, PeriodToken, Recurrence, Weekday};
pub use remind::RemindToken;
pub use time::TimeToken;

pub trait Token: Sized {
    fn parse(text: &str) -> Option<(Self, std::ops::Range<usize>)>;
}
