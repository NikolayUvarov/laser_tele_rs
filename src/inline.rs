use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::*;

/// Sent when the user typed "@your_bot query" in any chat.
/// Inline mode must be enabled for the bot with /setinline in @BotFather
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InlineQuery {
    pub id: String,
    pub from: User,
    pub query: String,
    /// `next_offset` of the previous answer, for paging
    pub offset: String,
    /// "sender", "private", "group", "supergroup" or "channel"
    pub chat_type: String,
    pub location: Option<Location>,
}

/// Sent when the user chose a result of an inline query.
/// These updates must be enabled for the bot with /setinlinefeedback in @BotFather
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChosenInlineResult {
    pub result_id: String,
    pub from: User,
    pub location: Option<Location>,
    /// Set if the message has an inline keyboard
    pub inline_message_id: String,
    pub query: String,
}

/// One result for `answer_inline_query` or `answer_guest_query`.
///
/// Create it with [`article`](Self::article), [`photo`](Self::photo) or [`cached`](Self::cached) and add
/// optional fields with [`set`](Self::set), or build any result type of
/// <https://core.telegram.org/bots/api#inlinequeryresult> from a JSON object:
///
/// ```
/// use laser_tele::{InlineQueryResult, json};
///
/// let article = InlineQueryResult::article("1", "Weather", "It's sunny").set("description", "Sunny, +25°C");
/// let venue = InlineQueryResult::from_json(json!({
///     "type": "venue", "id": "2", "title": "Office", "address": "Main street 1", "latitude": 55.75, "longitude": 37.62
/// }));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InlineQueryResult(pub Map<String, Value>);

impl InlineQueryResult {
    /// A result, which sends a text message
    pub fn article(id: &str, title: &str, message_text: &str) -> Self {
        Self::from_json(json!({
            "type": "article",
            "id": id,
            "title": title,
            "input_message_content": { "message_text": message_text },
        }))
    }

    /// A result, which sends a JPEG photo by URL
    pub fn photo(id: &str, photo_url: &str, thumbnail_url: &str) -> Self {
        Self::from_json(
            json!({ "type": "photo", "id": id, "photo_url": photo_url, "thumbnail_url": thumbnail_url }),
        )
    }

    /// A result, which sends a file already stored on Telegram servers by its file_id.
    /// `file_type` is "photo", "gif", "mpeg4_gif", "sticker", "document", "video", "voice" or "audio";
    /// `title` is required for "document", "video" and "voice"
    pub fn cached(file_type: &str, id: &str, file_id: &str, title: &str) -> Self {
        let field = if file_type == "mpeg4_gif" {
            "mpeg4_file_id".to_string()
        } else {
            format!("{file_type}_file_id")
        };
        let mut result = Map::new();
        result.insert("type".into(), json!(file_type));
        result.insert("id".into(), json!(id));
        result.insert(field, json!(file_id));
        if !title.is_empty() {
            result.insert("title".into(), json!(title));
        }
        InlineQueryResult(result)
    }

    /// A result from a JSON object with the fields of the result type
    pub fn from_json(value: Value) -> Self {
        match value {
            Value::Object(fields) => InlineQueryResult(fields),
            _ => InlineQueryResult::default(),
        }
    }

    /// Sets a field of the result, e.g. "description", "thumbnail_url", "reply_markup"
    pub fn set(mut self, name: &str, value: impl Serialize) -> Self {
        self.0.insert(
            name.to_string(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }
}

/// Optional parameters of `answer_inline_query`
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InlineQueryConfig {
    /// Seconds the results may be cached on the server, 300 by default
    pub cache_time: i64,
    /// The results are cached only for the user who sent the query
    pub is_personal: bool,
    /// Passed in the next query of the user ([`InlineQuery::offset`]) to get more results
    pub next_offset: String,
    /// A button above the results, which starts a private chat with the bot with `/start <button_start_parameter>`
    pub button_text: String,
    pub button_start_parameter: String,
}

impl Bot {
    /// Sends up to 50 results for the inline query
    pub async fn answer_inline_query(
        &self,
        inline_query_id: &str,
        results: &[InlineQueryResult],
        config: &InlineQueryConfig,
    ) -> Result<()> {
        let mut params = Map::new();
        params.insert("inline_query_id".into(), json!(inline_query_id));
        params.insert("results".into(), serde_json::to_value(results)?);
        if config.cache_time > 0 {
            params.insert("cache_time".into(), json!(config.cache_time));
        }
        if config.is_personal {
            params.insert("is_personal".into(), json!(true));
        }
        if !config.next_offset.is_empty() {
            params.insert("next_offset".into(), json!(config.next_offset));
        }
        if !config.button_text.is_empty() {
            params.insert(
                "button".into(),
                json!({ "text": config.button_text, "start_parameter": config.button_start_parameter }),
            );
        }
        self.call_json("answerInline", "answerInlineQuery", &Value::Object(params))
            .await
            .map(drop)
    }

    /// Answers a guest message ([`Message::guest_query_id`]) with the result
    pub async fn answer_guest_query(
        &self,
        guest_query_id: &str,
        result: &InlineQueryResult,
    ) -> Result<()> {
        let params = json!({ "guest_query_id": guest_query_id, "result": result });
        self.call_json("answerInline", "answerGuestQuery", &params)
            .await
            .map(drop)
    }
}
