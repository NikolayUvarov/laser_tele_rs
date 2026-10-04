//! Two bots in one program: the news bot and the support bot, each with its own token, logs and downloads.
//!
//! Run: NEWS_BOT_KEY=<token 1> SUPPORT_BOT_KEY=<token 2> cargo run --example several_bots

use std::time::Duration;

use laser_tele::{Bot, Config, UpdateKind};

#[tokio::main]
async fn main() -> laser_tele::Result<()> {
    let news = Bot::new(Config {
        api_key: std::env::var("NEWS_BOT_KEY").unwrap_or_default(),
        timeout: Duration::from_secs(5),
        log_dir: "logs/news".into(),
        download_dir: "files/news".into(),
        ..Default::default()
    })?;
    let support = Bot::new(Config {
        api_key: std::env::var("SUPPORT_BOT_KEY").unwrap_or_default(),
        timeout: Duration::from_secs(1),
        log_dir: "logs/support".into(),
        download_dir: "files/support".into(),
        ..Default::default()
    })?;

    let news_bot = news.run(|update| {
        let news = news.clone();
        async move {
            if let UpdateKind::Message(message) = &update.kind {
                report(
                    news.send_message(message.chat.id, "Today's news: the support bot works 24/7")
                        .await
                        .map(drop),
                );
            }
        }
    });
    let support_bot = support.run(|update| {
        let support = support.clone();
        async move {
            let UpdateKind::Message(message) = &update.kind else {
                return;
            };
            if let Some(document) = &message.document {
                match support.load_file(&document.file_id).await {
                    Ok(path) => println!("The support bot saved {}", path.display()),
                    Err(e) => report(Err(e)),
                }
            }
            report(
                support
                    .send_message(message.chat.id, "Your request is registered")
                    .await
                    .map(drop),
            );
        }
    });
    tokio::join!(news_bot, support_bot);
    Ok(())
}

fn report(result: laser_tele::Result<()>) {
    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}
