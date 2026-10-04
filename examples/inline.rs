//! Inline bot: type "@your_bot some text" in any chat and choose how to send the text.
//!
//! Enable inline mode with /setinline in @BotFather,
//! and /setinlinefeedback to get chosen_inline_result updates.
//!
//! Run: TG_API_KEY=<token> cargo run --example inline

use std::time::Duration;

use laser_tele::{Bot, Config, InlineQuery, InlineQueryConfig, InlineQueryResult, UpdateKind};

#[tokio::main]
async fn main() -> laser_tele::Result<()> {
    let bot = Bot::new(Config {
        timeout: Duration::from_secs(1),
        ..Default::default()
    })?;
    bot.run(|update| {
        let bot = bot.clone();
        async move {
            match &update.kind {
                UpdateKind::InlineQuery(query) => {
                    if let Err(e) = on_inline_query(&bot, query).await {
                        eprintln!("Error: {e}");
                    }
                }
                UpdateKind::ChosenInlineResult(result) => {
                    println!(
                        "{} sent result {} for query {}",
                        result.from.first_name, result.result_id, result.query
                    );
                }
                _ => {}
            }
        }
    })
    .await;
    Ok(())
}

async fn on_inline_query(bot: &Bot, query: &InlineQuery) -> laser_tele::Result<()> {
    let text = &query.query;
    if text.is_empty() {
        // a button above the results opens the private chat with the bot
        let config = InlineQueryConfig {
            button_text: "How to use the bot".into(),
            button_start_parameter: "help".into(),
            ..Default::default()
        };
        return bot.answer_inline_query(&query.id, &[], &config).await;
    }

    let reversed: String = text.chars().rev().collect();
    let results = vec![
        InlineQueryResult::article("upper", "UPPER CASE", &text.to_uppercase())
            .set("description", text.to_uppercase()),
        InlineQueryResult::article("lower", "lower case", &text.to_lowercase())
            .set("description", text.to_lowercase()),
        InlineQueryResult::article("reversed", "Reversed", &reversed).set("description", &reversed),
        // any result type of https://core.telegram.org/bots/api#inlinequeryresult can be built from JSON
        InlineQueryResult::photo(
            "photo",
            "https://telegram.org/img/t_logo.png",
            "https://telegram.org/img/t_logo.png",
        )
        .set("caption", text),
    ];
    let config = InlineQueryConfig {
        cache_time: 30,
        is_personal: true,
        ..Default::default()
    };
    bot.answer_inline_query(&query.id, &results, &config).await
}
