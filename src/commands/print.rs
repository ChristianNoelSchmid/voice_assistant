use async_trait::async_trait;

use super::CommandHandler;

/// Fallback handler that prints any unrecognized command to stdout.
///
/// Always matches, so it should be last in the handler list.
pub struct PrintHandler;

#[async_trait]
impl CommandHandler for PrintHandler {
    type Match = String;

    fn parse(&self, text: &str) -> Option<Self::Match> {
        Some(text.to_owned())
    }

    async fn handle(&mut self, matched: Self::Match) {
        println!("Command: {matched}");
    }
}
