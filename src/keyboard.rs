use serde::{Deserialize, Serialize};

fn is_false(value: &bool) -> bool {
    !*value
}

/// A keyboard attached to a message, see `send_keyboard`
///
/// ```
/// use laser_tele::{Button, InlineKeyboard};
///
/// let keyboard = InlineKeyboard::new(vec![
///     vec![Button::callback("Yes", "yes"), Button::callback("No", "no")],
///     vec![Button::url("Website", "https://example.com")],
/// ]);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InlineKeyboard {
    /// Rows of buttons
    #[serde(rename = "inline_keyboard")]
    pub rows: Vec<Vec<Button>>,
}

impl InlineKeyboard {
    /// A keyboard with rows of buttons; an empty keyboard removes the buttons of a message
    pub fn new(rows: Vec<Vec<Button>>) -> Self {
        InlineKeyboard { rows }
    }
}

/// A button of an inline keyboard. Create it with a constructor, or set `text` and one of the other fields
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Button {
    pub text: String,
    /// "danger" (red), "success" (green) or "primary" (blue)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub style: String,
    /// Sent to the bot in a callback query when the button is pressed, 1-64 bytes
    #[serde(skip_serializing_if = "String::is_empty")]
    pub callback_data: String,
    /// A link opened by the button
    #[serde(skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// Opens a chat chosen by the user with `"@your_bot <switch_inline_query>"` in the input field
    #[serde(skip_serializing_if = "String::is_empty")]
    pub switch_inline_query: String,
    /// Inserts `"@your_bot <switch_inline_query_current_chat>"` in the current chat
    #[serde(skip_serializing_if = "String::is_empty")]
    pub switch_inline_query_current_chat: String,
    /// Launches the game sent by `send_game`, the button must be the first one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_game: Option<CallbackGame>,
    /// The Pay button of an invoice, it must be the first one
    #[serde(skip_serializing_if = "is_false")]
    pub pay: bool,
}

/// A placeholder for [`Button::callback_game`]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CallbackGame {}

impl Button {
    /// A button sending `data` to the bot (a callback query) when pressed
    pub fn callback(text: impl Into<String>, data: impl Into<String>) -> Self {
        Button {
            text: text.into(),
            callback_data: data.into(),
            ..Default::default()
        }
    }

    /// A button opening a link
    pub fn url(text: impl Into<String>, url: impl Into<String>) -> Self {
        Button {
            text: text.into(),
            url: url.into(),
            ..Default::default()
        }
    }

    /// A button opening a chat chosen by the user with `"@your_bot <query>"` in the input field
    pub fn switch_inline_query(text: impl Into<String>, query: impl Into<String>) -> Self {
        Button {
            text: text.into(),
            switch_inline_query: query.into(),
            ..Default::default()
        }
    }

    /// A button inserting `"@your_bot <query>"` in the current chat
    pub fn switch_inline_query_current_chat(
        text: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        Button {
            text: text.into(),
            switch_inline_query_current_chat: query.into(),
            ..Default::default()
        }
    }

    /// The button launching the game sent by `send_game`
    pub fn game(text: impl Into<String>) -> Self {
        Button {
            text: text.into(),
            callback_game: Some(CallbackGame {}),
            ..Default::default()
        }
    }

    /// The Pay button of an invoice
    pub fn pay(text: impl Into<String>) -> Self {
        Button {
            text: text.into(),
            pay: true,
            ..Default::default()
        }
    }

    /// Sets the color of the button: "danger" (red), "success" (green) or "primary" (blue)
    pub fn style(mut self, style: impl Into<String>) -> Self {
        self.style = style.into();
        self
    }
}

/// Creates a button sending `callback` to the bot when pressed, the same as [`Button::callback`]
pub fn add_button(text: impl Into<String>, callback: impl Into<String>) -> Button {
    Button::callback(text, callback)
}
