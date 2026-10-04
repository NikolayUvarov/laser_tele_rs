use serde_json::json;

use crate::*;

impl Bot {
    /// Lets the user ([`ChatJoinRequest::from`]) join the chat
    pub async fn approve_chat_join_request(&self, chat_id: i64, user_id: i64) -> Result<()> {
        let params = json!({ "chat_id": chat_id, "user_id": user_id });
        self.call_json("joinRequests", "approveChatJoinRequest", &params)
            .await
            .map(drop)
    }

    /// Rejects the request of the user ([`ChatJoinRequest::from`]) to join the chat
    pub async fn decline_chat_join_request(&self, chat_id: i64, user_id: i64) -> Result<()> {
        let params = json!({ "chat_id": chat_id, "user_id": user_id });
        self.call_json("joinRequests", "declineChatJoinRequest", &params)
            .await
            .map(drop)
    }

    /// Returns the token of the bot ([`ManagedBotUpdated::bot`]) managed by this bot
    pub async fn get_managed_bot_token(&self, bot_user_id: i64) -> Result<String> {
        self.call_into(
            "managedBots",
            "getManagedBotToken",
            &json!({ "user_id": bot_user_id }),
        )
        .await
    }
}
