use super::CommandHandler;

/// Placeholder that prints any command until real handlers are wired up.
pub struct PrintHandler;

impl CommandHandler for PrintHandler {
    type Match = String;

    fn parse(&self, text: &str) -> Option<Self::Match> {
        Some(text.to_owned())
    }

    fn handle(&mut self, matched: Self::Match) {
        println!("Command: {matched}");
    }
}
