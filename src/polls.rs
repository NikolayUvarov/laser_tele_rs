use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::*;

/// A poll or a quiz ([`Message::poll`]), also sent as a poll update when its state changes
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Poll {
    pub id: String,
    pub question: String,
    pub question_entities: Vec<MessageEntity>,
    pub options: Vec<PollOption>,
    pub total_voter_count: i64,
    pub is_closed: bool,
    pub is_anonymous: bool,
    /// "regular" or "quiz"
    #[serde(rename = "type")]
    pub kind: String,
    pub allows_multiple_answers: bool,
    pub allows_revoting: bool,
    pub members_only: bool,
    /// For quizzes sent by the bot or closed
    pub correct_option_ids: Vec<i64>,
    pub explanation: String,
    pub explanation_entities: Vec<MessageEntity>,
    pub open_period: i64,
    pub close_date: i64,
    pub description: String,
}

/// An answer option of a poll with the number of votes
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PollOption {
    pub persistent_id: String,
    pub text: String,
    pub text_entities: Vec<MessageEntity>,
    pub voter_count: i64,
    pub added_by_user: Option<User>,
    pub added_by_chat: Option<Chat>,
    pub addition_date: i64,
}

/// Sent when a user voted in a non-anonymous poll sent by the bot
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PollAnswer {
    pub poll_id: String,
    /// For votes of anonymous chat administrators
    pub voter_chat: Option<Chat>,
    pub user: Option<User>,
    /// 0-based indexes of chosen options, empty if the vote was retracted
    pub option_ids: Vec<i64>,
    pub option_persistent_ids: Vec<String>,
}

/// Optional parameters of a poll for `send_poll`
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PollConfig {
    /// A quiz with correct answers instead of a regular poll
    pub quiz: bool,
    /// For a quiz: 0-based indexes of the correct options in increasing order
    pub correct_option_ids: Vec<i64>,
    /// For a quiz: shown when the user chose a wrong answer
    pub explanation: String,
    /// Votes are visible; the bot receives them as poll_answer updates only for such polls
    pub not_anonymous: bool,
    pub allows_multiple_answers: bool,
    /// Seconds the poll is active after creation, 5-600
    pub open_period: i64,
    /// Unix time when the poll is closed
    pub close_date: i64,
    pub is_closed: bool,
    pub reply_markup: Option<InlineKeyboard>,
}

impl Bot {
    /// Sends a poll with 1-12 options and returns the sent message (its `poll.id` identifies the poll)
    pub async fn send_poll(
        &self,
        chat_id: i64,
        question: &str,
        options: &[&str],
        config: &PollConfig,
    ) -> Result<Message> {
        let options: Vec<Value> = options
            .iter()
            .map(|option| json!({ "text": option }))
            .collect();
        let mut params = Map::new();
        params.insert("chat_id".into(), json!(chat_id));
        params.insert("question".into(), json!(question));
        params.insert("options".into(), Value::Array(options));
        if config.quiz {
            params.insert("type".into(), json!("quiz"));
            params.insert(
                "correct_option_ids".into(),
                json!(config.correct_option_ids),
            );
            if !config.explanation.is_empty() {
                params.insert("explanation".into(), json!(config.explanation));
            }
        }
        if config.not_anonymous {
            params.insert("is_anonymous".into(), json!(false));
        }
        if config.allows_multiple_answers {
            params.insert("allows_multiple_answers".into(), json!(true));
        }
        if config.open_period > 0 {
            params.insert("open_period".into(), json!(config.open_period));
        }
        if config.close_date > 0 {
            params.insert("close_date".into(), json!(config.close_date));
        }
        if config.is_closed {
            params.insert("is_closed".into(), json!(true));
        }
        if let Some(keyboard) = &config.reply_markup {
            params.insert("reply_markup".into(), serde_json::to_value(keyboard)?);
        }
        self.call_into("polls", "sendPoll", &Value::Object(params))
            .await
    }

    /// Closes the poll sent by the bot and returns its final results
    pub async fn stop_poll(&self, chat_id: i64, message_id: i64) -> Result<Poll> {
        self.call_into(
            "polls",
            "stopPoll",
            &json!({ "chat_id": chat_id, "message_id": message_id }),
        )
        .await
    }
}
