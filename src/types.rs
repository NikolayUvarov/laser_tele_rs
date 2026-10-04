//! Telegram Bot API objects (<https://core.telegram.org/bots/api#available-types>).
//!
//! Optional objects are `Option`; absent strings, numbers and lists are empty,
//! e.g. a message has text if `!message.text.is_empty()`.
//! The field `type` of Telegram is called `kind`. Fields not described here can be read from [`Update::raw`](crate::Update::raw).

use serde::{Deserialize, Serialize};

use crate::InlineKeyboard;

/// A Telegram user or bot
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub language_code: String,
}

/// A private chat, group, supergroup or channel
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Chat {
    pub id: i64,
    /// "private", "group", "supergroup" or "channel"
    #[serde(rename = "type")]
    pub kind: String,
    /// For groups and channels
    pub title: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub is_forum: bool,
}

/// A special part of the text: a command, a link, bold text...
/// `offset` and `length` are in UTF-16 code units, see [`Message::entity_text`]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageEntity {
    /// "bot_command", "mention", "url", "bold", "text_link"...
    #[serde(rename = "type")]
    pub kind: String,
    pub offset: i64,
    pub length: i64,
    /// For "text_link"
    pub url: String,
    /// For "text_mention"
    pub user: Option<User>,
    pub language: String,
    pub custom_emoji_id: String,
}

/// One size of a photo or a thumbnail
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i64,
    pub height: i64,
    pub file_size: i64,
}

/// A video file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Video {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i64,
    pub height: i64,
    pub duration: i64,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
}

/// A GIF or H.264/MPEG-4 AVC video without sound
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Animation {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i64,
    pub height: i64,
    pub duration: i64,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
}

/// A music file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Audio {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i64,
    pub performer: String,
    pub title: String,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
}

/// A general file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Document {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
}

/// A voice note
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Voice {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i64,
    pub mime_type: String,
    pub file_size: i64,
}

/// A round video message
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VideoNote {
    pub file_id: String,
    pub file_unique_id: String,
    pub length: i64,
    pub duration: i64,
    pub file_size: i64,
}

/// A sticker
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sticker {
    pub file_id: String,
    pub file_unique_id: String,
    /// "regular", "mask" or "custom_emoji"
    #[serde(rename = "type")]
    pub kind: String,
    pub width: i64,
    pub height: i64,
    pub is_animated: bool,
    pub is_video: bool,
    pub emoji: String,
    pub set_name: String,
    pub file_size: i64,
}

/// A phone contact
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Contact {
    pub phone_number: String,
    pub first_name: String,
    pub last_name: String,
    pub user_id: i64,
    pub vcard: String,
}

/// A point on the map
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub horizontal_accuracy: f64,
    pub live_period: i64,
}

/// A place with a name and an address
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Venue {
    pub location: Location,
    pub title: String,
    pub address: String,
    pub foursquare_id: String,
    pub foursquare_type: String,
    pub google_place_id: String,
    pub google_place_type: String,
}

/// An animated emoji with a random value
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Dice {
    pub emoji: String,
    pub value: i64,
}

/// Data sent by a Web App to the bot
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebAppData {
    pub data: String,
    pub button_text: String,
}

/// The origin of a forwarded message
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageOrigin {
    /// "user", "hidden_user", "chat" or "channel"
    #[serde(rename = "type")]
    pub kind: String,
    pub date: i64,
    /// For "user"
    pub sender_user: Option<User>,
    /// For "hidden_user"
    pub sender_user_name: String,
    /// For "chat"
    pub sender_chat: Option<Chat>,
    /// For "channel"
    pub chat: Option<Chat>,
    /// For "channel"
    pub message_id: i64,
    pub author_signature: String,
}

/// A message of any kind: text, photo, poll, payment, service message...
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Message {
    pub message_id: i64,
    /// Topic in a forum supergroup
    pub message_thread_id: i64,
    /// Empty for messages in channels
    pub from: Option<User>,
    /// For messages sent on behalf of a chat
    pub sender_chat: Option<Chat>,
    pub chat: Chat,
    pub date: i64,
    pub edit_date: i64,
    /// Set for messages of a connected business account, pass it to
    /// [`MessageConfig::business_connection_id`](crate::MessageConfig::business_connection_id) to answer on behalf of the account
    pub business_connection_id: String,
    /// Set for guest messages, answer them with `answer_guest_query`
    pub guest_query_id: String,
    pub forward_origin: Option<MessageOrigin>,
    pub reply_to_message: Option<Box<Message>>,
    /// The bot, via which the message was sent in inline mode
    pub via_bot: Option<User>,
    pub is_topic_message: bool,
    pub has_protected_content: bool,
    /// The same for photos and videos sent as an album
    pub media_group_id: String,
    pub author_signature: String,
    pub effect_id: String,
    pub text: String,
    pub entities: Vec<MessageEntity>,
    /// Text of a photo, video, document...
    pub caption: String,
    pub caption_entities: Vec<MessageEntity>,
    /// Sizes of the photo, the last one is the largest
    pub photo: Vec<PhotoSize>,
    pub video: Option<Video>,
    pub animation: Option<Animation>,
    pub audio: Option<Audio>,
    pub document: Option<Document>,
    pub voice: Option<Voice>,
    pub video_note: Option<VideoNote>,
    pub sticker: Option<Sticker>,
    pub contact: Option<Contact>,
    pub dice: Option<Dice>,
    pub game: Option<crate::Game>,
    pub poll: Option<crate::Poll>,
    pub venue: Option<Venue>,
    pub location: Option<Location>,
    pub new_chat_members: Vec<User>,
    pub left_chat_member: Option<User>,
    pub new_chat_title: String,
    pub new_chat_photo: Vec<PhotoSize>,
    pub delete_chat_photo: bool,
    /// The group is upgraded to a supergroup with this ID, use it instead of `chat.id`
    pub migrate_to_chat_id: i64,
    pub migrate_from_chat_id: i64,
    pub pinned_message: Option<Box<Message>>,
    pub invoice: Option<crate::Invoice>,
    /// The user paid an invoice, deliver the goods
    pub successful_payment: Option<crate::SuccessfulPayment>,
    pub refunded_payment: Option<crate::RefundedPayment>,
    pub web_app_data: Option<WebAppData>,
    pub connected_website: String,
    pub reply_markup: Option<InlineKeyboard>,
}

impl Message {
    /// The command of the message, like "/start" for "/start@your_bot promo", or "" if the message is not a command
    pub fn command(&self) -> &str {
        match self.entities.first() {
            Some(entity) if entity.kind == "bot_command" && entity.offset == 0 => {
                let command = self.text.split_whitespace().next().unwrap_or("");
                command.split('@').next().unwrap_or("")
            }
            _ => "",
        }
    }

    /// The text of an entity of the text (offsets of entities are in UTF-16 code units)
    pub fn entity_text(&self, entity: &MessageEntity) -> String {
        let units: Vec<u16> = self.text.encode_utf16().collect();
        let start = (entity.offset.max(0) as usize).min(units.len());
        let end = ((entity.offset + entity.length).max(0) as usize).clamp(start, units.len());
        String::from_utf16_lossy(&units[start..end])
    }
}

/// Sent when the user pressed an inline button. Answer it with `answer_callback_query`,
/// otherwise Telegram shows progress on the button
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    /// The message with the pressed button
    pub message: Option<Message>,
    pub inline_message_id: String,
    pub chat_instance: String,
    /// `callback_data` of the pressed button
    pub data: String,
    /// The "Play" button of a game was pressed
    pub game_short_name: String,
}

/// A member of a chat; fields of other statuses are empty
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatMember {
    /// "creator", "administrator", "member", "restricted", "left" or "kicked"
    pub status: String,
    pub user: User,
    pub is_anonymous: bool,
    pub custom_title: String,
    pub until_date: i64,
    /// For "restricted"
    pub is_member: bool,
    pub can_be_edited: bool,
    pub can_manage_chat: bool,
    pub can_delete_messages: bool,
    pub can_restrict_members: bool,
    pub can_promote_members: bool,
    pub can_change_info: bool,
    pub can_invite_users: bool,
    pub can_pin_messages: bool,
    pub can_post_messages: bool,
    pub can_edit_messages: bool,
    pub can_send_messages: bool,
}

/// Sent when the status of a member in a chat is changed, e.g. the user blocked the bot
/// (`new_chat_member.status == "kicked"`) or the bot was added to a group
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatMemberUpdated {
    pub chat: Chat,
    pub from: User,
    pub date: i64,
    pub old_chat_member: ChatMember,
    pub new_chat_member: ChatMember,
    pub invite_link: Option<ChatInviteLink>,
    pub via_join_request: bool,
    pub via_chat_folder_invite_link: bool,
}

/// An invite link to a chat
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatInviteLink {
    pub invite_link: String,
    pub creator: User,
    pub creates_join_request: bool,
    pub is_primary: bool,
    pub is_revoked: bool,
    pub name: String,
    pub expire_date: i64,
    pub member_limit: i64,
    pub pending_join_request_count: i64,
    pub subscription_period: i64,
    pub subscription_price: i64,
}

/// Sent when a user asked to join the chat, answer it with `approve_chat_join_request` or
/// `decline_chat_join_request`. The bot must be an administrator with the right to invite users
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatJoinRequest {
    pub chat: Chat,
    pub from: User,
    /// The private chat with the user, the bot can write there for 5 minutes
    pub user_chat_id: i64,
    pub date: i64,
    pub bio: String,
    pub invite_link: Option<ChatInviteLink>,
    pub query_id: String,
}

/// The source of a chat boost
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatBoostSource {
    /// "premium", "gift_code" or "giveaway"
    pub source: String,
    pub user: Option<User>,
    pub giveaway_message_id: i64,
    pub prize_star_count: i64,
    pub is_unclaimed: bool,
}

/// A boost added to a chat
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatBoost {
    pub boost_id: String,
    pub add_date: i64,
    pub expiration_date: i64,
    pub source: ChatBoostSource,
}

/// Sent when a boost was added to a chat or changed
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatBoostUpdated {
    pub chat: Chat,
    pub boost: ChatBoost,
}

/// Sent when a boost was removed from a chat
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatBoostRemoved {
    pub chat: Chat,
    pub boost_id: String,
    pub remove_date: i64,
    pub source: ChatBoostSource,
}

/// Rights of the bot in a connected business account
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BusinessBotRights {
    pub can_reply: bool,
    pub can_read_messages: bool,
    pub can_delete_sent_messages: bool,
    pub can_delete_all_messages: bool,
    pub can_edit_name: bool,
    pub can_edit_bio: bool,
    pub can_edit_profile_photo: bool,
    pub can_edit_username: bool,
    pub can_change_gift_settings: bool,
    pub can_view_gifts_and_stars: bool,
    pub can_convert_gifts_to_stars: bool,
    pub can_transfer_and_upgrade_gifts: bool,
    pub can_transfer_stars: bool,
    pub can_manage_stories: bool,
}

/// Sent when the bot was connected to or disconnected from a business account. Messages of the account
/// come as business_message updates, answer them with `send_message_with_config` and
/// [`MessageConfig::business_connection_id`](crate::MessageConfig::business_connection_id)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BusinessConnection {
    pub id: String,
    pub user: User,
    pub user_chat_id: i64,
    pub date: i64,
    pub rights: Option<BusinessBotRights>,
    pub is_enabled: bool,
}

/// Sent when messages were deleted in a connected business account
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BusinessMessagesDeleted {
    pub business_connection_id: String,
    pub chat: Chat,
    pub message_ids: Vec<i64>,
}

/// Sent when a bot managed by this bot was created or its token or owner was changed,
/// get its token with `get_managed_bot_token`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ManagedBotUpdated {
    /// The user who created the bot
    pub user: User,
    pub bot: User,
}

/// Sent when the user asked the bot to stop generation of a message
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageGenerationStopped {
    pub chat: Chat,
    pub message_thread_id: i64,
    pub draft_id: i64,
}

/// A file on Telegram servers, the result of getFile
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct File {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_size: i64,
    pub file_path: String,
}
