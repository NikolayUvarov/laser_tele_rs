//! laser_tele is a library for Telegram bots (<https://core.telegram.org/bots/api>),
//! the Rust version of the Go library [laser_tele](https://github.com/NikolayUvarov/laser_tele).
//!
//! It covers the whole flow of a bot: receiving updates, answering messages and buttons, files, polls,
//! inline mode, payments in Telegram Stars, games, reactions, group management and business accounts.
//! Bot API methods without their own function can be called with `call`.
//!
//! # Two APIs
//!
//! The asynchronous API (tokio) is the crate root: [`Bot`] and functions of the default bot like [`send_message`].
//! The blocking API is [`blocking`]: [`blocking::Bot`] and the same functions, without `async`.
//!
//! # Quick start, blocking
//!
//! Create a bot with @BotFather, put its token to the `TG_API_KEY` environment variable
//! (or to the `.APIKEY` file) and run:
//!
//! ```no_run
//! use laser_tele::{blocking, UpdateKind};
//!
//! fn main() -> laser_tele::Result<()> {
//!     blocking::run(|update| {
//!         if let UpdateKind::Message(message) = &update.kind {
//!             if let Err(e) = blocking::send_message(message.chat.id, &format!("You said: {}", message.text)) {
//!                 eprintln!("Can't send message: {e}");
//!             }
//!         }
//!     })
//! }
//! ```
//!
//! # Quick start, async
//!
//! ```no_run
//! use laser_tele::UpdateKind;
//!
//! #[tokio::main]
//! async fn main() -> laser_tele::Result<()> {
//!     laser_tele::run(|update| async move {
//!         if let UpdateKind::Message(message) = &update.kind {
//!             if let Err(e) = laser_tele::send_message(message.chat.id, &format!("You said: {}", message.text)).await {
//!                 eprintln!("Can't send message: {e}");
//!             }
//!         }
//!     })
//!     .await
//! }
//! ```
//!
//! # The default bot and several bots
//!
//! Functions of the crate root and of [`blocking`] work with the default bot, configured by [`init`]
//! or created on the first call with the token from the environment. To run several bots in one program,
//! create each with [`Bot::new`] (or [`blocking::Bot::new`]): a bot has the same methods.
//!
//! # Updates
//!
//! `run` requests updates every [`Config::timeout`] and passes each of them to [`Config::on_update`],
//! to the channel created by `make_chan` and to the handler. [`Update::kind`] is the kind of the update
//! with its object, [`Update::type_name`] its name ("message", "callback_query", "poll_answer"...),
//! [`Update::raw`] the JSON received from Telegram.
//!
//! # Errors and logs
//!
//! Functions sending requests return [`Result`]; when Telegram refused the request the error is [`Error::Api`].
//! Requests and responses are written to `*.log` files without texts of messages and without the token,
//! see [`LogMode`], [`Config::log_dir`] and [`Config::log_max_size`].

mod bot;
mod chats;
mod config;
mod error;
mod games;
mod inline;
mod keyboard;
mod logger;
mod messages;
mod payments;
mod polls;
mod reactions;
mod types;
mod update;

pub use bot::Bot;
pub use config::{Config, LogMode, UpdateCallback};
pub use error::{ApiError, Error, Result};
pub use games::{Game, GameHighScore, GameMessage};
pub use inline::{ChosenInlineResult, InlineQuery, InlineQueryConfig, InlineQueryResult};
pub use keyboard::{Button, CallbackGame, InlineKeyboard, add_button};
pub use messages::{CallbackAnswerConfig, MessageConfig};
pub use payments::{
    BotSubscriptionUpdated, Invoice, InvoiceConfig, LabeledPrice, OrderInfo, PaidMediaPurchased,
    PreCheckoutQuery, RefundedPayment, ShippingAddress, ShippingOption, ShippingQuery,
    SuccessfulPayment,
};
pub use polls::{Poll, PollAnswer, PollConfig, PollOption};
pub use reactions::{
    MessageReactionCountUpdated, MessageReactionUpdated, ReactionCount, ReactionType,
};
pub use serde_json::{Value, json};
pub use types::*;
pub use update::{ALL_UPDATE_TYPES, Update, UpdateKind};

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tokio::sync::mpsc;

/// Calls `$callback!` with the list of methods, which exist on `Bot`, `blocking::Bot`
/// and as functions of the default bot
macro_rules! bot_methods {
    ($callback:ident) => {
        $callback! {
            /// Sends a text message to the chat and returns the sent message. The text is sent as is, up to 4096 characters
            fn send_message(chat_id: i64, text: &str) -> Message;
            /// Sends a message with formatting, a reply, a keyboard and other options and returns the sent message
            fn send_message_with_config(chat_id: i64, text: &str, config: &MessageConfig) -> Message;
            /// Changes the text of a message sent by the bot. Without a keyboard Telegram removes the buttons of the message
            fn edit_message_text(chat_id: i64, message_id: i64, text: &str) -> Message;
            /// Changes the inline keyboard of a message sent by the bot, an empty keyboard removes it
            fn edit_message_reply_markup(chat_id: i64, message_id: i64, keyboard: &InlineKeyboard) -> Message;
            /// Sends a message with text and an inline keyboard
            fn send_keyboard(chat_id: i64, text: &str, keyboard: &InlineKeyboard) -> Message;
            /// Answers the pressed inline button, otherwise Telegram shows progress on the button.
            /// `text` (optional) is shown to the user as a notification
            fn answer_callback_query(callback_query_id: &str, text: &str) -> ();
            /// Answers the pressed inline button with optional parameters: an alert, a URL of a game...
            fn answer_callback_query_with_config(callback_query_id: &str, config: &CallbackAnswerConfig) -> ();
            /// Uploads a local photo file to the chat, `caption` is the text under it (up to 1024 characters)
            fn send_photo(chat_id: i64, caption: &str, path: impl AsRef<Path> + Send) -> Message;
            /// Uploads a local video file to the chat, `caption` is the text under it
            fn send_video(chat_id: i64, caption: &str, path: impl AsRef<Path> + Send) -> Message;
            /// Uploads a local file to the chat as a document, `caption` is the text under it
            fn send_document(chat_id: i64, caption: &str, path: impl AsRef<Path> + Send) -> Message;
            /// Downloads a file sent by a user (its `file_id`) to `Config::download_dir` and returns its path.
            /// Bots can download files up to 20 MB
            fn load_file(file_id: &str) -> PathBuf;
            /// Downloads the file by `link` and saves it to `file_path` in `Config::download_dir`
            fn file_download(link: &str, file_path: &str) -> PathBuf;
            /// Sends a poll with 1-12 options and returns the sent message (its `poll.id` identifies the poll)
            fn send_poll(chat_id: i64, question: &str, options: &[&str], config: &PollConfig) -> Message;
            /// Closes the poll sent by the bot and returns its final results
            fn stop_poll(chat_id: i64, message_id: i64) -> Poll;
            /// Sends up to 50 results for the inline query
            fn answer_inline_query(inline_query_id: &str, results: &[InlineQueryResult], config: &InlineQueryConfig) -> ();
            /// Answers a guest message (`Message::guest_query_id`) with the result
            fn answer_guest_query(guest_query_id: &str, result: &InlineQueryResult) -> ();
            /// Sends an invoice and returns the sent message
            fn send_invoice(chat_id: i64, invoice: &InvoiceConfig) -> Message;
            /// Creates a link to pay the invoice, it can be sent anywhere
            fn create_invoice_link(invoice: &InvoiceConfig) -> String;
            /// Answers the shipping query with ways of delivery.
            /// A non-empty `error_message` (shown to the user) means that the delivery is impossible
            fn answer_shipping_query(shipping_query_id: &str, options: &[ShippingOption], error_message: &str) -> ();
            /// Confirms the payment (empty `error_message`) or cancels it with `error_message` shown to the user.
            /// It must be called within 10 seconds after the query is received
            fn answer_pre_checkout_query(pre_checkout_query_id: &str, error_message: &str) -> ();
            /// Returns Telegram Stars of the payment (`SuccessfulPayment::telegram_payment_charge_id`) to the user
            fn refund_star_payment(user_id: i64, telegram_payment_charge_id: &str) -> ();
            /// Cancels (`is_canceled` true) or re-enables the subscription of the user paid in Telegram Stars
            fn edit_user_star_subscription(user_id: i64, telegram_payment_charge_id: &str, is_canceled: bool) -> ();
            /// Sends the game created in @BotFather and returns the sent message
            fn send_game(chat_id: i64, game_short_name: &str) -> Message;
            /// Sets the score of the user in the game. A score lower than the current one is set only with `force`
            fn set_game_score(user_id: i64, score: i64, message: &GameMessage, force: bool) -> ();
            /// Returns the high scores of the user and several of their neighbors in the game
            fn get_game_high_scores(user_id: i64, message: &GameMessage) -> Vec<GameHighScore>;
            /// Sets the reaction of the bot to a message, an empty emoji removes it.
            /// Only emoji allowed for reactions can be used: "👍", "👎", "❤", "🔥", "🎉", "👏"...
            fn set_message_reaction(chat_id: i64, message_id: i64, emoji: &str) -> ();
            /// Lets the user join the chat (`ChatJoinRequest::from`)
            fn approve_chat_join_request(chat_id: i64, user_id: i64) -> ();
            /// Rejects the request of the user to join the chat (`ChatJoinRequest::from`)
            fn decline_chat_join_request(chat_id: i64, user_id: i64) -> ();
            /// Returns the token of the bot (`ManagedBotUpdated::bot`) managed by this bot
            fn get_managed_bot_token(bot_user_id: i64) -> String;
            /// Calls any Bot API method (<https://core.telegram.org/bots/api#available-methods>) with `params`
            /// (a JSON object) and returns the `result` field of the response
            fn call(method: &str, params: Value) -> Value;
        }
    };
}

macro_rules! define_async_functions {
    ($( $(#[$attr:meta])* fn $name:ident($($arg:ident : $ty:ty),*) -> $ret:ty; )*) => {
        $(
            $(#[$attr])*
            ///
            /// Works with the default bot, see [`init`]
            pub async fn $name($($arg: $ty),*) -> Result<$ret> {
                default_bot()?.$name($($arg),*).await
            }
        )*
    };
}

pub mod blocking;

static DEFAULT_BOT: Mutex<Option<Bot>> = Mutex::new(None);

/// Configures the default bot, used by the functions of the crate root and of [`blocking`]
pub fn init(config: Config) -> Result<()> {
    let bot = Bot::new(config)?;
    *bot::lock(&DEFAULT_BOT) = Some(bot);
    blocking::reset_default_bot();
    Ok(())
}

/// The default bot; if [`init`] wasn't called, it is created with the token from `TG_API_KEY` or `.APIKEY`
pub fn default_bot() -> Result<Bot> {
    let mut default = bot::lock(&DEFAULT_BOT);
    if default.is_none() {
        *default = Some(Bot::new(Config::default())?);
    }
    Ok(default.clone().expect("the default bot is set"))
}

/// Requests updates of the default bot every `Config::timeout` and passes each of them to `Config::on_update`,
/// to the channel (if [`make_chan`] was called) and to `handler`. It returns only an error of creating the bot
pub async fn run<F, Fut>(handler: F) -> Result<()>
where
    F: FnMut(Update) -> Fut,
    Fut: Future<Output = ()>,
{
    default_bot()?.run(handler).await;
    Ok(())
}

/// Requests new updates of the default bot once and passes each of them like [`run`]
pub async fn update_request<F, Fut>(handler: F) -> Result<()>
where
    F: FnMut(Update) -> Fut,
    Fut: Future<Output = ()>,
{
    default_bot()?.update_request(handler).await;
    Ok(())
}

/// Creates the channel, to which updates of the default bot are sent, see [`Bot::make_chan`]
pub fn make_chan() -> Result<mpsc::Receiver<Update>> {
    Ok(default_bot()?.make_chan())
}

bot_methods!(define_async_functions);

// the code of README.md is compiled by `cargo test --doc`
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
