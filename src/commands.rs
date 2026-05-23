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
        if let Some(m) = self.parse(text) {
            self.handle(m);
            true
        } else {
            false
        }
    }
}

/// Placeholder that prints any command until real handlers are wired up.
pub struct PrintHandler;

impl CommandHandler for PrintHandler {
    type Match = String;

    fn parse(&self, text: &str) -> Option<Self::Match> {
        if(text.contains("print")) {
            Some(text.to_owned())
        } else {
            None
        }
    }

    fn handle(&mut self, matched: Self::Match) {
        println!("Command: {matched}");
    }
}
