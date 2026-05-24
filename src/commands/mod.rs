mod print;
mod remind;

pub use print::PrintHandler;
pub use remind::RemindCommand;

use async_trait::async_trait;

use crate::tokens;

#[async_trait]
pub trait CommandHandler {
    type Match: Send;

    fn parse(&self, text: &str) -> Option<Self::Match>;
    async fn handle(&mut self, matched: Self::Match);
}

#[async_trait]
pub trait DynCommandHandler {
    async fn try_handle(&mut self, text: &str) -> bool;
}

#[async_trait]
impl<H: CommandHandler + Send + Sync> DynCommandHandler for H {
    async fn try_handle(&mut self, text: &str) -> bool {
        let normalized = tokens::normalize(text);
        if let Some(m) = self.parse(&normalized) {
            self.handle(m).await;
            true
        } else {
            false
        }
    }
}
