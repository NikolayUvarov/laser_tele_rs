//! Polls bot: /poll sends a poll, /quiz sends a quiz, /stop closes the last poll and shows the results.
//! Votes come to the bot only for non-anonymous polls sent by it.
//!
//! Run: TG_API_KEY=<token> cargo run --example polls

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use laser_tele::{Bot, Config, Message, PollConfig, UpdateKind};

#[tokio::main]
async fn main() -> laser_tele::Result<()> {
    let bot = Bot::new(Config::default())?;
    // the last poll in every chat: chat ID -> message ID
    let last_polls = Arc::new(Mutex::new(HashMap::new()));

    bot.run(|update| {
        let (bot, last_polls) = (bot.clone(), last_polls.clone());
        async move {
            let result = match &update.kind {
                UpdateKind::Message(message) => on_command(&bot, &last_polls, message).await,
                UpdateKind::PollAnswer(answer) => {
                    let name = answer
                        .user
                        .as_ref()
                        .map(|user| user.first_name.as_str())
                        .unwrap_or("Anonymous");
                    if answer.option_ids.is_empty() {
                        println!("{name} retracted the vote in poll {}", answer.poll_id);
                    } else {
                        println!(
                            "{name} voted for options {:?} in poll {}",
                            answer.option_ids, answer.poll_id
                        );
                    }
                    Ok(())
                }
                UpdateKind::Poll(poll) => {
                    // a new state of a poll sent by the bot: numbers of votes, closing
                    println!(
                        "Poll {} has {} votes, closed: {}",
                        poll.id, poll.total_voter_count, poll.is_closed
                    );
                    Ok(())
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

async fn on_command(
    bot: &Bot,
    last_polls: &Mutex<HashMap<i64, i64>>,
    message: &Message,
) -> laser_tele::Result<()> {
    let chat_id = message.chat.id;
    let sent = match message.text.as_str() {
        "/poll" => {
            let config = PollConfig {
                not_anonymous: true,
                allows_multiple_answers: true,
                ..Default::default()
            };
            bot.send_poll(
                chat_id,
                "Which languages do you use?",
                &["Rust", "Go", "Python", "Other"],
                &config,
            )
            .await?
        }
        "/quiz" => {
            let config = PollConfig {
                quiz: true,
                correct_option_ids: vec![1],
                explanation: "tokio::spawn runs a future as a new task".into(),
                not_anonymous: true,
                open_period: 60,
                ..Default::default()
            };
            bot.send_poll(
                chat_id,
                "Which function starts a task?",
                &["std::thread::sleep", "tokio::spawn"],
                &config,
            )
            .await?
        }
        "/stop" => {
            let message_id = last_polls.lock().unwrap().remove(&chat_id);
            let Some(message_id) = message_id else {
                bot.send_message(chat_id, "There is no poll to stop, send /poll or /quiz")
                    .await?;
                return Ok(());
            };
            let poll = bot.stop_poll(chat_id, message_id).await?;
            let mut results = vec![format!("Results of \"{}\":", poll.question)];
            results.extend(
                poll.options
                    .iter()
                    .map(|option| format!("{}: {}", option.text, option.voter_count)),
            );
            bot.send_message(chat_id, &results.join("\n")).await?;
            return Ok(());
        }
        _ => return Ok(()),
    };
    last_polls.lock().unwrap().insert(chat_id, sent.message_id);
    Ok(())
}
