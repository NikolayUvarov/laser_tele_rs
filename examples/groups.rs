//! Group bot: approves join requests, greets new members, reacts to messages, reports reactions and boosts.
//!
//! Add the bot to a group as an administrator with the right to invite users:
//! Telegram sends chat_member, reactions and boosts only to administrators.
//!
//! Run: TG_API_KEY=<token> cargo run --example groups

use laser_tele::{Bot, Config, Update, UpdateKind};

#[tokio::main]
async fn main() -> laser_tele::Result<()> {
    let bot = Bot::new(Config::default())?;
    bot.run(|update| {
        let bot = bot.clone();
        async move {
            if let Err(e) = on_update(&bot, &update).await {
                eprintln!("Error: {e}");
            }
        }
    })
    .await;
    Ok(())
}

async fn on_update(bot: &Bot, update: &Update) -> laser_tele::Result<()> {
    match &update.kind {
        UpdateKind::ChatJoinRequest(request) => {
            // approve everybody except users without a username
            if request.from.username.is_empty() {
                bot.decline_chat_join_request(request.chat.id, request.from.id)
                    .await?;
            } else {
                bot.approve_chat_join_request(request.chat.id, request.from.id)
                    .await?;
            }
        }
        UpdateKind::ChatMember(change) => {
            if change.old_chat_member.status == "left" && change.new_chat_member.status == "member"
            {
                let name = &change.new_chat_member.user.first_name;
                bot.send_message(change.chat.id, &format!("Welcome, {name}!"))
                    .await?;
            }
        }
        UpdateKind::MyChatMember(change) => {
            // the bot was added to or removed from a chat, or blocked by a user in a private chat
            println!(
                "The bot is {} in {} {:?}",
                change.new_chat_member.status, change.chat.kind, change.chat.title
            );
        }
        UpdateKind::Message(message) => {
            if message.chat.kind != "private" && message.text.to_lowercase().contains("thank") {
                bot.set_message_reaction(message.chat.id, message.message_id, "❤")
                    .await?;
            }
            if message.migrate_to_chat_id != 0 {
                println!(
                    "The group {} is now the supergroup {}",
                    message.chat.id, message.migrate_to_chat_id
                );
            }
        }
        UpdateKind::MessageReaction(reaction) => {
            let name = reaction
                .user
                .as_ref()
                .map(|user| user.first_name.as_str())
                .unwrap_or("Anonymous");
            for r in &reaction.new_reaction {
                println!(
                    "{name} reacted with {} to message {}",
                    r.emoji, reaction.message_id
                );
            }
        }
        UpdateKind::MessageReactionCount(counts) => {
            // reactions of anonymous administrators and in channels come only as counts
            for count in &counts.reactions {
                println!("{} {}", count.reaction.emoji, count.total_count);
            }
        }
        UpdateKind::ChatBoost(boost) => {
            let name = boost
                .boost
                .source
                .user
                .as_ref()
                .map(|user| user.first_name.as_str())
                .unwrap_or("friend");
            bot.send_message(boost.chat.id, &format!("Thank you for the boost, {name}!"))
                .await?;
        }
        _ => {}
    }
    Ok(())
}
