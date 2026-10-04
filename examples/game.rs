//! Game bot: /game sends an HTML5 game, the "Play" button opens it, the game reports scores to this program.
//!
//!  1. Create a game with /newgame in @BotFather, its short name is GAME_SHORT_NAME.
//!  2. Host the game page at GAME_URL. When the user finishes, the page calls
//!     <this server>:8080/score?user=<user>&chat=<chat>&message=<message>&inline=<inline>&score=<score>
//!     with the parameters this bot added to GAME_URL.
//!
//! Run: TG_API_KEY=<token> GAME_SHORT_NAME=race GAME_URL=https://example.com/race cargo run --example game

use std::collections::HashMap;

use axum::Router;
use axum::extract::Query;
use axum::routing::get;
use laser_tele::{
    Bot, CallbackAnswerConfig, CallbackQuery, Config, GameMessage, Message, UpdateKind,
};

#[tokio::main]
async fn main() -> laser_tele::Result<()> {
    let bot = Bot::new(Config::default())?;
    let short_name = std::env::var("GAME_SHORT_NAME").unwrap_or_default();
    let game_url = std::env::var("GAME_URL").unwrap_or_default();

    // the server receiving scores from the game page
    let score_bot = bot.clone();
    let app = Router::new().route(
        "/score",
        get(move |Query(query): Query<HashMap<String, String>>| {
            let bot = score_bot.clone();
            async move { on_score(&bot, &query).await }
        }),
    );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tokio::spawn(async move { axum::serve(listener, app).await });

    bot.run(|update| {
        let (bot, short_name, game_url) = (bot.clone(), short_name.clone(), game_url.clone());
        async move {
            let result = match &update.kind {
                UpdateKind::Message(message) if message.text == "/game" => {
                    bot.send_game(message.chat.id, &short_name).await.map(drop)
                }
                UpdateKind::Message(message) if message.text == "/top" => {
                    send_top(&bot, message).await
                }
                UpdateKind::CallbackQuery(query) if !query.game_short_name.is_empty() => {
                    open_game(&bot, query, &game_url).await
                }
                _ => Ok(()),
            };
            if let Err(e) = result {
                eprintln!("Error: {e}");
            }
        }
    })
    .await;
    Ok(())
}

/// Answers the "Play" button with the URL of the game, which knows the player and the message
async fn open_game(bot: &Bot, query: &CallbackQuery, game_url: &str) -> laser_tele::Result<()> {
    let target = match &query.message {
        _ if !query.inline_message_id.is_empty() => format!("inline={}", query.inline_message_id),
        Some(message) => format!("chat={}&message={}", message.chat.id, message.message_id),
        None => String::new(),
    };
    let url = format!("{game_url}?user={}&{target}", query.from.id);
    bot.answer_callback_query_with_config(
        &query.id,
        &CallbackAnswerConfig {
            url,
            ..Default::default()
        },
    )
    .await
}

/// Receives the score from the game page and saves it in Telegram
async fn on_score(bot: &Bot, query: &HashMap<String, String>) -> String {
    let number = |name: &str| {
        query
            .get(name)
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0)
    };
    let game = match query.get("inline") {
        Some(inline_message_id) => GameMessage::Inline(inline_message_id.clone()),
        None => GameMessage::Chat {
            chat_id: number("chat"),
            message_id: number("message"),
        },
    };
    // a real game must check that the score is not faked, e.g. sign the parameters
    match bot
        .set_game_score(number("user"), number("score"), &game, false)
        .await
    {
        Ok(()) => "ok".into(),
        Err(e) => e.to_string(),
    }
}

/// Shows the high scores of the game the command replies to
async fn send_top(bot: &Bot, message: &Message) -> laser_tele::Result<()> {
    let chat_id = message.chat.id;
    let Some(game_message) = message
        .reply_to_message
        .as_ref()
        .filter(|reply| reply.game.is_some())
    else {
        bot.send_message(chat_id, "Reply with /top to a message with the game")
            .await?;
        return Ok(());
    };
    let user_id = message.from.as_ref().map(|user| user.id).unwrap_or(0);
    let game = GameMessage::Chat {
        chat_id,
        message_id: game_message.message_id,
    };
    let scores = bot.get_game_high_scores(user_id, &game).await?;
    let mut lines = vec!["Top players:".to_string()];
    lines.extend(
        scores
            .iter()
            .map(|row| format!("{}. {} — {}", row.position, row.user.first_name, row.score)),
    );
    bot.send_message(chat_id, &lines.join("\n")).await?;
    Ok(())
}
