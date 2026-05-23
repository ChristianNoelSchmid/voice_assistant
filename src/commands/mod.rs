mod print;
mod remind;

pub use print::PrintHandler;
pub use remind::RemindCommand;

use crate::tokens;

pub trait CommandHandler {
    type Match;

    fn parse(&self, text: &str) -> Option<Self::Match>;
    fn handle(&mut self, matched: Self::Match);
}

pub trait DynCommandHandler {
    fn try_handle(&mut self, text: &str) -> bool;
}

impl<H: CommandHandler> DynCommandHandler for H {
    fn try_handle(&mut self, text: &str) -> bool {
        let normalized = tokens::normalize(text);
        if let Some(m) = self.parse(&normalized) {
            self.handle(m);
            true
        } else {
            false
        }
    }
}
