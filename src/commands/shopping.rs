use std::sync::LazyLock;

use async_trait::async_trait;
use regex::Regex;

use crate::speaker::DynSpeaker;
use crate::tasks::DynTaskClient;

use super::CommandHandler;

/// Parsed item extracted from a "add X to my shopping list" utterance.
pub struct ShoppingMatch {
    pub item: String,
}

/// Handles "add [item] to my (grocery|shopping)[ list]" utterances by adding an item to Vikunja.
pub struct ShoppingCommand {
    client: DynTaskClient,
    speaker: DynSpeaker,
    project_id: u64,
}

impl ShoppingCommand {
    /// Create a new [`ShoppingCommand`] that writes items into `project_id`.
    pub fn new(client: DynTaskClient, speaker: DynSpeaker, project_id: u64) -> Self {
        Self {
            client,
            speaker,
            project_id,
        }
    }
}

// Matches: "add <item> to my (grocery/groceries/shopping)[ list]"
static SHOPPING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(add|put|and) (.+) (and|to|in)( (my|the))? (grocer(y|ies)|shopping)( list)?")
        .unwrap()
});

#[async_trait]
impl CommandHandler for ShoppingCommand {
    type Match = ShoppingMatch;

    fn parse(&self, text: &str) -> Option<Self::Match> {
        let caps = SHOPPING_RE.captures(text)?;
        // Group 1 is the leading verb (add/put/and); group 2 is the item.
        let item = caps[2].split_whitespace().collect::<Vec<_>>().join(" ");
        Some(ShoppingMatch { item })
    }

    async fn handle(&self, matched: Self::Match) {
        println!("[Shopping] \"{}\"", matched.item);
        match self
            .client
            .create_task(&matched.item, None, None, None, self.project_id)
            .await
        {
            Ok(()) => {
                println!("[Shopping] Item added.");
                let text = format!("Added \"{}\" to your shopping list.", matched.item);
                self.speaker.speak(text).await.unwrap();
            }
            Err(e) => eprintln!("[Shopping] Failed to add item: {e}"),
        }
    }
}
