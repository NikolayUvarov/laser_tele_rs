//! Updates processed by several worker threads: updates are read from a channel,
//! so a slow answer to one user doesn't delay the others.
//!
//! Run: TG_API_KEY=<token> cargo run --example channel

use std::sync::{Arc, Mutex};
use std::time::Duration;

use laser_tele::{UpdateKind, blocking};

fn main() -> laser_tele::Result<()> {
    // must be called before run
    let updates = Arc::new(Mutex::new(blocking::make_chan()?));

    for worker in 1..=4 {
        let updates = updates.clone();
        std::thread::spawn(move || {
            loop {
                let update = match updates.lock().unwrap().recv() {
                    Ok(update) => update,
                    Err(_) => return,
                };
                if let UpdateKind::Message(message) = &update.kind {
                    println!("Worker {worker} processes message {}", message.message_id);
                    std::thread::sleep(Duration::from_secs(3)); // a slow job
                    if let Err(e) =
                        blocking::send_message(message.chat.id, &format!("Done: {}", message.text))
                    {
                        eprintln!("Error: {e}");
                    }
                }
            }
        });
    }

    // updates go to the channel, the handler does nothing
    blocking::run(|_| {})
}
