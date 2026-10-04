//! Echo bot on the blocking API: answers every text message with the same text.
//!
//! Run: TG_API_KEY=<token from @BotFather> cargo run --example echo

use laser_tele::{UpdateKind, blocking};

fn main() -> laser_tele::Result<()> {
    blocking::run(|update| {
        if let UpdateKind::Message(message) = &update.kind {
            if message.text.is_empty() {
                return;
            }
            if let Err(e) = blocking::send_message(message.chat.id, &message.text) {
                eprintln!("Can't send message: {e}");
            }
        }
    })
}
