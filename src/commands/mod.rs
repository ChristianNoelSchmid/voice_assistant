mod clock;
mod remind;
mod shopping;
mod unhandled;

pub use clock::ClockCommand;
pub use remind::RemindCommand;
pub use shopping::ShoppingCommand;
pub use unhandled::UnhandledCommand;

use async_trait::async_trait;

use crate::tokens;

/// Typed command handler. `parse` extracts a strongly-typed match from normalized text;
/// `handle` acts on it. Automatically gets a [`DynCommandHandler`] blanket impl.
#[async_trait]
pub trait CommandHandler {
    type Match: Send;

    fn parse(&self, text: &str) -> Option<Self::Match>;
    async fn handle(&self, matched: Self::Match);
}

// CommandHandler has an associated type and cannot be used as a trait object directly.
// DynCommandHandler is the object-safe wrapper stored in the dispatcher's handler list.
/// Object-safe wrapper around [`CommandHandler`] for use in heterogeneous handler lists.
#[async_trait]
pub trait DynCommandHandler: Send + Sync {
    /// Normalize `text`, attempt to parse it, and handle the result.
    /// Returns `true` if the command was claimed and handled.
    async fn try_handle(&self, text: &str) -> bool;
}

#[async_trait]
impl<H: CommandHandler + Send + Sync> DynCommandHandler for H {
    async fn try_handle(&self, text: &str) -> bool {
        let normalized = tokens::normalize(text);
        if let Some(m) = self.parse(&normalized) {
            self.handle(m).await;
            true
        } else {
            false
        }
    }
}
