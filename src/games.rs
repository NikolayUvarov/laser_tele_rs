//! Games: <https://core.telegram.org/bots/games>. A game is created with /newgame in @BotFather.
//! When the user presses the "Play" button, a callback query with `game_short_name` is sent,
//! answer it with `answer_callback_query_with_config` and the URL of the game

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::*;

/// A game sent by `send_game` ([`Message::game`])
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Game {
    pub title: String,
    pub description: String,
    pub photo: Vec<PhotoSize>,
    pub text: String,
    pub text_entities: Vec<MessageEntity>,
    pub animation: Option<Animation>,
}

/// One row of the high scores table of a game
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameHighScore {
    pub position: i64,
    pub user: User,
    pub score: i64,
}

/// The message with a game
#[derive(Debug, Clone, PartialEq)]
pub enum GameMessage {
    /// A game sent to a chat
    Chat { chat_id: i64, message_id: i64 },
    /// A game sent in inline mode ([`CallbackQuery::inline_message_id`])
    Inline(String),
}

impl GameMessage {
    fn params(&self, user_id: i64) -> Value {
        match self {
            GameMessage::Chat {
                chat_id,
                message_id,
            } => {
                json!({ "user_id": user_id, "chat_id": chat_id, "message_id": message_id })
            }
            GameMessage::Inline(inline_message_id) => {
                json!({ "user_id": user_id, "inline_message_id": inline_message_id })
            }
        }
    }
}

impl Bot {
    /// Sends the game created in @BotFather and returns the sent message
    pub async fn send_game(&self, chat_id: i64, game_short_name: &str) -> Result<Message> {
        self.call_into(
            "games",
            "sendGame",
            &json!({ "chat_id": chat_id, "game_short_name": game_short_name }),
        )
        .await
    }

    /// Sets the score of the user in the game. A score lower than the current one is set only with `force`
    pub async fn set_game_score(
        &self,
        user_id: i64,
        score: i64,
        message: &GameMessage,
        force: bool,
    ) -> Result<()> {
        let mut params = message.params(user_id);
        params["score"] = json!(score);
        if force {
            params["force"] = json!(true);
        }
        self.call_json("games", "setGameScore", &params)
            .await
            .map(drop)
    }

    /// Returns the high scores of the user and several of their neighbors in the game
    pub async fn get_game_high_scores(
        &self,
        user_id: i64,
        message: &GameMessage,
    ) -> Result<Vec<GameHighScore>> {
        self.call_into("games", "getGameHighScores", &message.params(user_id))
            .await
    }
}
