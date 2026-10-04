//! Files bot: saves photos, videos, documents and voice notes sent by users to downloadedFiles/
//! and sends a document back by the /report command.
//!
//! Run: TG_API_KEY=<token> cargo run --example files

use laser_tele::{Config, Message, UpdateKind, blocking};

fn main() -> laser_tele::Result<()> {
    let bot = blocking::Bot::new(Config {
        download_dir: "downloadedFiles".into(),
        ..Default::default()
    })?;
    bot.run(|update| {
        if let UpdateKind::Message(message) = &update.kind
            && let Err(e) = on_message(&bot, message)
        {
            eprintln!("Error: {e}");
        }
    });
    Ok(())
}

fn on_message(bot: &blocking::Bot, message: &Message) -> laser_tele::Result<()> {
    let chat_id = message.chat.id;

    if message.text == "/report" {
        std::fs::write("report.txt", "Report of the bot")?;
        bot.send_document(chat_id, "Your report", "report.txt")?;
        return Ok(());
    }

    let file_id = if let Some(largest) = message.photo.last() {
        // sizes of the photo, the last one is the largest
        &largest.file_id
    } else if let Some(video) = &message.video {
        &video.file_id
    } else if let Some(document) = &message.document {
        &document.file_id
    } else if let Some(voice) = &message.voice {
        &voice.file_id
    } else {
        bot.send_message(
            chat_id,
            "Send me a photo, video, document or voice note, or /report",
        )?;
        return Ok(());
    };

    // bots can download files up to 20 MB
    let path = bot.load_file(file_id)?;
    let mut text = format!("Saved to {}", path.display());
    if !message.caption.is_empty() {
        text += &format!(" with caption: {}", message.caption);
    }
    bot.send_message(chat_id, &text)?;
    Ok(())
}
