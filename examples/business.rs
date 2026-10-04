//! Business bot: answers customers on behalf of a Telegram Business account while its owner is away.
//!
//! The owner connects the bot in Telegram: Settings > Telegram Business > Chatbots.
//!
//! Run: TG_API_KEY=<token> cargo run --example business

use laser_tele::{Config, MessageConfig, UpdateKind, blocking};

fn main() -> laser_tele::Result<()> {
    let bot = blocking::Bot::new(Config::default())?;
    bot.run(|update| {
        let result = match &update.kind {
            UpdateKind::BusinessConnection(connection) => {
                let can_reply = connection
                    .rights
                    .as_ref()
                    .is_some_and(|rights| rights.can_reply);
                println!(
                    "Business account of {} enabled: {}, can reply: {can_reply}",
                    connection.user.first_name, connection.is_enabled
                );
                Ok(())
            }
            UpdateKind::BusinessMessage(message) => {
                // messages written by the owner of the account come too, answer only customers
                let from_customer = message
                    .from
                    .as_ref()
                    .is_some_and(|user| user.id == message.chat.id);
                if from_customer {
                    let config = MessageConfig {
                        business_connection_id: message.business_connection_id.clone(),
                        reply_to_message_id: message.message_id,
                        ..Default::default()
                    };
                    bot.send_message_with_config(
                        message.chat.id,
                        "Thank you for your message! We will answer within an hour.",
                        &config,
                    )
                    .map(drop)
                } else {
                    Ok(())
                }
            }
            UpdateKind::DeletedBusinessMessages(deleted) => {
                println!(
                    "Messages {:?} were deleted in chat {}",
                    deleted.message_ids, deleted.chat.id
                );
                Ok(())
            }
            _ => Ok(()),
        };
        if let Err(e) = result {
            eprintln!("Error: {e}");
        }
    });
    Ok(())
}
