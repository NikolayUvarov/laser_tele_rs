use serde_json::{Map, Value, json};

use crate::*;

/// Optional parameters of a message for `send_message_with_config`
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MessageConfig {
    /// "HTML", "MarkdownV2" or "Markdown"
    pub parse_mode: String,
    /// The message is a reply to this message
    pub reply_to_message_id: i64,
    /// Topic in a forum supergroup
    pub message_thread_id: i64,
    pub disable_notification: bool,
    /// The message can't be forwarded and saved
    pub protect_content: bool,
    /// [`Message::business_connection_id`] sends the message on behalf of a connected business account
    pub business_connection_id: String,
    pub reply_markup: Option<InlineKeyboard>,
}

/// Optional parameters of `answer_callback_query_with_config`
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CallbackAnswerConfig {
    /// A notification for the user
    pub text: String,
    /// Show the text as an alert instead of a notification at the top of the chat
    pub show_alert: bool,
    /// A URL opened by the user's client, e.g. the game for the "Play" button of a game
    pub url: String,
    /// Seconds the answer may be cached on the client
    pub cache_time: i64,
}

impl Bot {
    /// Sends a text message to the chat and returns the sent message. The text is sent as is, up to 4096 characters
    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<Message> {
        self.call_into(
            "sendMessage",
            "sendMessage",
            &json!({ "chat_id": chat_id, "text": text }),
        )
        .await
    }

    /// Sends a message with formatting, a reply, a keyboard and other options and returns the sent message
    pub async fn send_message_with_config(
        &self,
        chat_id: i64,
        text: &str,
        config: &MessageConfig,
    ) -> Result<Message> {
        let mut params = Map::new();
        params.insert("chat_id".into(), json!(chat_id));
        params.insert("text".into(), json!(text));
        if !config.parse_mode.is_empty() {
            params.insert("parse_mode".into(), json!(config.parse_mode));
        }
        if config.reply_to_message_id != 0 {
            params.insert(
                "reply_parameters".into(),
                json!({ "message_id": config.reply_to_message_id }),
            );
        }
        if config.message_thread_id != 0 {
            params.insert("message_thread_id".into(), json!(config.message_thread_id));
        }
        if config.disable_notification {
            params.insert("disable_notification".into(), json!(true));
        }
        if config.protect_content {
            params.insert("protect_content".into(), json!(true));
        }
        if !config.business_connection_id.is_empty() {
            params.insert(
                "business_connection_id".into(),
                json!(config.business_connection_id),
            );
        }
        if let Some(keyboard) = &config.reply_markup {
            params.insert("reply_markup".into(), serde_json::to_value(keyboard)?);
        }
        self.call_into("sendMessage", "sendMessage", &Value::Object(params))
            .await
    }

    /// Changes the text of a message sent by the bot. Without a keyboard Telegram removes the buttons of the message
    pub async fn edit_message_text(
        &self,
        chat_id: i64,
        message_id: i64,
        text: &str,
    ) -> Result<Message> {
        let params = json!({ "chat_id": chat_id, "message_id": message_id, "text": text });
        self.call_into("editMessage", "editMessageText", &params)
            .await
    }

    /// Changes the inline keyboard of a message sent by the bot, an empty keyboard removes it
    pub async fn edit_message_reply_markup(
        &self,
        chat_id: i64,
        message_id: i64,
        keyboard: &InlineKeyboard,
    ) -> Result<Message> {
        let params =
            json!({ "chat_id": chat_id, "message_id": message_id, "reply_markup": keyboard });
        self.call_into("editMessage", "editMessageReplyMarkup", &params)
            .await
    }

    /// Sends a message with text and an inline keyboard
    pub async fn send_keyboard(
        &self,
        chat_id: i64,
        text: &str,
        keyboard: &InlineKeyboard,
    ) -> Result<Message> {
        let params = json!({ "chat_id": chat_id, "text": text, "reply_markup": keyboard });
        self.call_into("sendKeyboard", "sendMessage", &params).await
    }

    /// Answers the pressed inline button ([`CallbackQuery::id`]), otherwise Telegram shows progress on the button.
    /// `text` (optional) is shown to the user as a notification
    pub async fn answer_callback_query(&self, callback_query_id: &str, text: &str) -> Result<()> {
        let config = CallbackAnswerConfig {
            text: text.to_string(),
            ..Default::default()
        };
        self.answer_callback_query_with_config(callback_query_id, &config)
            .await
    }

    /// Answers the pressed inline button like `answer_callback_query`, with optional parameters
    pub async fn answer_callback_query_with_config(
        &self,
        callback_query_id: &str,
        config: &CallbackAnswerConfig,
    ) -> Result<()> {
        let mut params = Map::new();
        params.insert("callback_query_id".into(), json!(callback_query_id));
        if !config.text.is_empty() {
            params.insert("text".into(), json!(config.text));
        }
        if config.show_alert {
            params.insert("show_alert".into(), json!(true));
        }
        if !config.url.is_empty() {
            params.insert("url".into(), json!(config.url));
        }
        if config.cache_time > 0 {
            params.insert("cache_time".into(), json!(config.cache_time));
        }
        self.call_json(
            "answerCallback",
            "answerCallbackQuery",
            &Value::Object(params),
        )
        .await
        .map(drop)
    }
}
