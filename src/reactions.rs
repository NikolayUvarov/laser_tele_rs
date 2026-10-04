//! Telegram sends updates about reactions only if the bot is an administrator in the chat

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::*;

/// A reaction: an emoji, a custom emoji or a paid reaction
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReactionType {
    /// "emoji", "custom_emoji" or "paid"
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub emoji: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub custom_emoji_id: String,
}

/// The number of reactions of one type
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReactionCount {
    #[serde(rename = "type")]
    pub reaction: ReactionType,
    pub total_count: i64,
}

/// Sent when a user changed their reaction to a message
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageReactionUpdated {
    pub chat: Chat,
    pub message_id: i64,
    pub user: Option<User>,
    /// For anonymous reactions on behalf of a chat
    pub actor_chat: Option<Chat>,
    pub date: i64,
    pub old_reaction: Vec<ReactionType>,
    pub new_reaction: Vec<ReactionType>,
}

/// Sent when anonymous reactions to a message were changed
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageReactionCountUpdated {
    pub chat: Chat,
    pub message_id: i64,
    pub date: i64,
    pub reactions: Vec<ReactionCount>,
}

impl Bot {
    /// Sets the reaction of the bot to a message, an empty emoji removes it.
    /// Only emoji allowed for reactions can be used: "👍", "👎", "❤", "🔥", "🎉", "👏"...
    pub async fn set_message_reaction(
        &self,
        chat_id: i64,
        message_id: i64,
        emoji: &str,
    ) -> Result<()> {
        let reaction = if emoji.is_empty() {
            vec![]
        } else {
            vec![ReactionType {
                kind: "emoji".into(),
                emoji: emoji.into(),
                ..Default::default()
            }]
        };
        let params = json!({ "chat_id": chat_id, "message_id": message_id, "reaction": reaction });
        self.call_json("reactions", "setMessageReaction", &params)
            .await
            .map(drop)
    }
}
