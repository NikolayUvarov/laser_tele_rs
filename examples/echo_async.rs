//! Echo bot on the asynchronous API: answers every text message with the same text.
//!
//! Run: TG_API_KEY=<token from @BotFather> cargo run --example echo_async

use std::time::Duration;

use laser_tele::{Config, UpdateKind};

#[tokio::main]
async fn main() -> laser_tele::Result<()> {
    laser_tele::init(Config {
        timeout: Duration::from_secs(2),
        ..Default::default()
    })?;
    laser_tele::run(|update| async move {
        if let UpdateKind::Message(message) = &update.kind {
            if message.text.is_empty() {
                return;
            }
            if let Err(e) = laser_tele::send_message(message.chat.id, &message.text).await {
                eprintln!("Can't send message: {e}");
            }
        }
    })
    .await
}
