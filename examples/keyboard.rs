//! Keyboard bot: /menu shows a message with buttons; pressing a button changes the message.
//!
//! Run: TG_API_KEY=<token> cargo run --example keyboard

use laser_tele::{
    Button, CallbackAnswerConfig, CallbackQuery, Config, InlineKeyboard, UpdateKind, blocking,
};

fn menu() -> InlineKeyboard {
    InlineKeyboard::new(vec![
        vec![
            Button::callback("☕ Coffee", "coffee"),
            Button::callback("🍵 Tea", "tea"),
        ],
        vec![Button::callback("❌ Cancel", "cancel").style("danger")],
        vec![Button::url("🌐 Website", "https://core.telegram.org/bots")],
    ])
}

fn main() -> laser_tele::Result<()> {
    let bot = blocking::Bot::new(Config::default())?;
    bot.run(|update| {
        let result = match &update.kind {
            UpdateKind::Message(message) if message.text == "/menu" => bot
                .send_keyboard(message.chat.id, "What would you like?", &menu())
                .map(drop),
            UpdateKind::CallbackQuery(query) => on_button(&bot, query),
            _ => Ok(()),
        };
        if let Err(e) = result {
            eprintln!("Error: {e}");
        }
    });
    Ok(())
}

fn on_button(bot: &blocking::Bot, query: &CallbackQuery) -> laser_tele::Result<()> {
    let Some(message) = &query.message else {
        return bot.answer_callback_query(&query.id, "");
    };
    let (chat_id, message_id) = (message.chat.id, message.message_id);

    // the pressed button must be answered, otherwise Telegram shows progress on it
    if query.data == "cancel" {
        let config = CallbackAnswerConfig {
            text: "Order canceled".into(),
            show_alert: true,
            ..Default::default()
        };
        bot.answer_callback_query_with_config(&query.id, &config)?;
        // remove the buttons
        bot.edit_message_reply_markup(chat_id, message_id, &InlineKeyboard::default())?;
        bot.edit_message_text(chat_id, message_id, "Nothing ordered")?;
        return Ok(());
    }

    bot.answer_callback_query(&query.id, &format!("You chose {}", query.data))?;
    bot.edit_message_text(
        chat_id,
        message_id,
        &format!("Your order: {}. It will be ready in 5 minutes", query.data),
    )?;
    Ok(())
}
