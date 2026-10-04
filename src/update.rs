use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::*;

/// All kinds of updates, the bot receives them by default ([`Config::allowed_updates`])
pub const ALL_UPDATE_TYPES: &[&str] = &[
    "message",
    "edited_message",
    "channel_post",
    "edited_channel_post",
    "business_connection",
    "business_message",
    "edited_business_message",
    "deleted_business_messages",
    "guest_message",
    "message_reaction",
    "message_reaction_count",
    "inline_query",
    "chosen_inline_result",
    "callback_query",
    "shipping_query",
    "pre_checkout_query",
    "purchased_paid_media",
    "poll",
    "poll_answer",
    "my_chat_member",
    "chat_member",
    "chat_join_request",
    "chat_boost",
    "removed_chat_boost",
    "managed_bot",
    "subscription",
    "stopped_message_generation",
];

/// An incoming update: a message, a pressed button, a vote, a payment...
///
/// ```
/// # fn handle(update: laser_tele::Update) {
/// use laser_tele::UpdateKind;
///
/// match &update.kind {
///     UpdateKind::Message(message) => println!("Message: {}", message.text),
///     UpdateKind::CallbackQuery(query) => println!("Button: {}", query.data),
///     _ => println!("Skipped update of type {}", update.type_name()),
/// }
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub update_id: i64,
    pub kind: UpdateKind,
    /// The update as received from Telegram, for fields not described in the library
    pub raw: Value,
}

/// The kind of an update with its object
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum UpdateKind {
    /// A new message of any kind
    Message(Message),
    /// A message was edited
    EditedMessage(Message),
    /// A new post in a channel, the bot must be an administrator of the channel
    ChannelPost(Message),
    /// A post was edited
    EditedChannelPost(Message),
    /// The bot was connected to or disconnected from a business account
    BusinessConnection(BusinessConnection),
    /// A message in a connected business account
    BusinessMessage(Message),
    /// A message in a connected business account was edited
    EditedBusinessMessage(Message),
    /// Messages were deleted in a connected business account
    DeletedBusinessMessages(BusinessMessagesDeleted),
    /// A guest message, answer it with `answer_guest_query`
    GuestMessage(Message),
    /// A user changed a reaction, the bot must be an administrator
    MessageReaction(MessageReactionUpdated),
    /// Anonymous reactions changed, the bot must be an administrator
    MessageReactionCount(MessageReactionCountUpdated),
    /// "@your_bot text" was typed in a chat
    InlineQuery(InlineQuery),
    /// A result of an inline query was sent
    ChosenInlineResult(ChosenInlineResult),
    /// An inline button was pressed, answer it with `answer_callback_query`
    CallbackQuery(CallbackQuery),
    /// The user entered the address for an invoice with `is_flexible`
    ShippingQuery(ShippingQuery),
    /// The user confirmed the payment, answer within 10 seconds with `answer_pre_checkout_query`
    PreCheckoutQuery(PreCheckoutQuery),
    /// Paid media sent by the bot was bought
    PurchasedPaidMedia(PaidMediaPurchased),
    /// A poll sent by the bot changed
    Poll(Poll),
    /// A user voted in a non-anonymous poll sent by the bot
    PollAnswer(PollAnswer),
    /// The status of the bot changed: added to a group, blocked by a user...
    MyChatMember(ChatMemberUpdated),
    /// The status of a member changed, the bot must be an administrator
    ChatMember(ChatMemberUpdated),
    /// A user asked to join a chat
    ChatJoinRequest(ChatJoinRequest),
    /// A chat was boosted
    ChatBoost(ChatBoostUpdated),
    /// A boost was removed
    RemovedChatBoost(ChatBoostRemoved),
    /// A bot managed by this bot was created or changed
    ManagedBot(ManagedBotUpdated),
    /// A paid subscription was changed
    Subscription(BotSubscriptionUpdated),
    /// The user asked to stop generating a message
    StoppedMessageGeneration(MessageGenerationStopped),
    /// A kind not described in the library, with its name; read it from [`Update::raw`]
    Unknown(String),
}

impl Update {
    /// Creates the update from the JSON received from Telegram
    pub fn from_value(raw: Value) -> Update {
        let update_id = raw.get("update_id").and_then(Value::as_i64).unwrap_or(0);
        let name = raw
            .as_object()
            .and_then(|fields| {
                fields
                    .keys()
                    .find(|name| name.as_str() != "update_id")
                    .cloned()
            })
            .unwrap_or_default();
        let value = raw.get(&name).cloned().unwrap_or(Value::Null);

        fn parse<T: DeserializeOwned>(value: Value) -> Option<T> {
            serde_json::from_value(value).ok()
        }
        use UpdateKind as K;
        let kind = match name.as_str() {
            "message" => parse(value).map(K::Message),
            "edited_message" => parse(value).map(K::EditedMessage),
            "channel_post" => parse(value).map(K::ChannelPost),
            "edited_channel_post" => parse(value).map(K::EditedChannelPost),
            "business_connection" => parse(value).map(K::BusinessConnection),
            "business_message" => parse(value).map(K::BusinessMessage),
            "edited_business_message" => parse(value).map(K::EditedBusinessMessage),
            "deleted_business_messages" => parse(value).map(K::DeletedBusinessMessages),
            "guest_message" => parse(value).map(K::GuestMessage),
            "message_reaction" => parse(value).map(K::MessageReaction),
            "message_reaction_count" => parse(value).map(K::MessageReactionCount),
            "inline_query" => parse(value).map(K::InlineQuery),
            "chosen_inline_result" => parse(value).map(K::ChosenInlineResult),
            "callback_query" => parse(value).map(K::CallbackQuery),
            "shipping_query" => parse(value).map(K::ShippingQuery),
            "pre_checkout_query" => parse(value).map(K::PreCheckoutQuery),
            "purchased_paid_media" => parse(value).map(K::PurchasedPaidMedia),
            "poll" => parse(value).map(K::Poll),
            "poll_answer" => parse(value).map(K::PollAnswer),
            "my_chat_member" => parse(value).map(K::MyChatMember),
            "chat_member" => parse(value).map(K::ChatMember),
            "chat_join_request" => parse(value).map(K::ChatJoinRequest),
            "chat_boost" => parse(value).map(K::ChatBoost),
            "removed_chat_boost" => parse(value).map(K::RemovedChatBoost),
            "managed_bot" => parse(value).map(K::ManagedBot),
            "subscription" => parse(value).map(K::Subscription),
            "stopped_message_generation" => parse(value).map(K::StoppedMessageGeneration),
            _ => None,
        }
        .unwrap_or(K::Unknown(name));
        Update {
            update_id,
            kind,
            raw,
        }
    }

    /// The kind of the update, the name of its field in the Bot API:
    /// "message", "callback_query", "poll", "inline_query"... (see [`ALL_UPDATE_TYPES`])
    pub fn type_name(&self) -> &str {
        use UpdateKind as K;
        match &self.kind {
            K::Message(_) => "message",
            K::EditedMessage(_) => "edited_message",
            K::ChannelPost(_) => "channel_post",
            K::EditedChannelPost(_) => "edited_channel_post",
            K::BusinessConnection(_) => "business_connection",
            K::BusinessMessage(_) => "business_message",
            K::EditedBusinessMessage(_) => "edited_business_message",
            K::DeletedBusinessMessages(_) => "deleted_business_messages",
            K::GuestMessage(_) => "guest_message",
            K::MessageReaction(_) => "message_reaction",
            K::MessageReactionCount(_) => "message_reaction_count",
            K::InlineQuery(_) => "inline_query",
            K::ChosenInlineResult(_) => "chosen_inline_result",
            K::CallbackQuery(_) => "callback_query",
            K::ShippingQuery(_) => "shipping_query",
            K::PreCheckoutQuery(_) => "pre_checkout_query",
            K::PurchasedPaidMedia(_) => "purchased_paid_media",
            K::Poll(_) => "poll",
            K::PollAnswer(_) => "poll_answer",
            K::MyChatMember(_) => "my_chat_member",
            K::ChatMember(_) => "chat_member",
            K::ChatJoinRequest(_) => "chat_join_request",
            K::ChatBoost(_) => "chat_boost",
            K::RemovedChatBoost(_) => "removed_chat_boost",
            K::ManagedBot(_) => "managed_bot",
            K::Subscription(_) => "subscription",
            K::StoppedMessageGeneration(_) => "stopped_message_generation",
            K::Unknown(name) => name,
        }
    }

    /// The message of a "message" update
    pub fn message(&self) -> Option<&Message> {
        match &self.kind {
            UpdateKind::Message(message) => Some(message),
            _ => None,
        }
    }

    /// The pressed button of a "callback_query" update
    pub fn callback_query(&self) -> Option<&CallbackQuery> {
        match &self.kind {
            UpdateKind::CallbackQuery(query) => Some(query),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Update {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Ok(Update::from_value(Value::deserialize(deserializer)?))
    }
}

impl Serialize for Update {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}
