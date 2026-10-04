//! The blocking API: the same bot and functions, without `async`.
//!
//! Requests are executed on an internal tokio runtime, so don't call these functions inside async code,
//! use the asynchronous API there.
//!
//! ```no_run
//! use laser_tele::{blocking, Config, UpdateKind};
//!
//! fn main() -> laser_tele::Result<()> {
//!     let bot = blocking::Bot::new(Config::default())?;
//!     bot.run(|update| {
//!         if let UpdateKind::CallbackQuery(query) = &update.kind {
//!             let _ = bot.answer_callback_query(&query.id, "Pressed");
//!         }
//!     });
//!     Ok(())
//! }
//! ```

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex, OnceLock};

use crate::bot::lock;
use crate::*;

fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("laser_tele")
            .enable_all()
            .build()
            .expect("laser_tele: can't start the tokio runtime")
    })
}

fn block_on<F: Future>(future: F) -> F::Output {
    runtime().block_on(future)
}

/// A Telegram bot with the blocking API. Several bots with different tokens can work in one program.
/// `Bot` is cheap to clone: clones share the connection pool and the state of the bot
#[derive(Clone)]
pub struct Bot {
    inner: crate::Bot,
    updates_tx: Arc<Mutex<Option<SyncSender<Update>>>>,
}

impl Bot {
    /// Creates a bot, see [`crate::Bot::new`]
    pub fn new(config: Config) -> Result<Bot> {
        Ok(Bot::from_async(crate::Bot::new(config)?))
    }

    /// The blocking API of the asynchronous bot; both share the state of the bot
    pub fn from_async(bot: crate::Bot) -> Bot {
        Bot {
            inner: bot,
            updates_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// The asynchronous API of the bot
    pub fn as_async(&self) -> &crate::Bot {
        &self.inner
    }

    /// Creates the channel, to which updates are sent, and returns its receiver. It must be called before `run`.
    /// Updates must be read from the channel, otherwise processing of updates waits
    pub fn make_chan(&self) -> Receiver<Update> {
        let (tx, rx) = sync_channel(0);
        *lock(&self.updates_tx) = Some(tx);
        rx
    }

    /// Requests updates every `Config::timeout` and passes each of them to `Config::on_update`,
    /// to the channel (if `make_chan` was called) and to `handler`. It never returns.
    ///
    /// Updates sent before the start of the bot are skipped, so the bot doesn't answer old messages.
    /// While the handler works, new updates wait: start a thread for slow jobs
    pub fn run<F: FnMut(Update)>(&self, mut handler: F) {
        loop {
            self.update_request(&mut handler);
            std::thread::sleep(self.inner.timeout());
        }
    }

    /// Requests new updates once and passes each of them like `run`
    pub fn update_request<F: FnMut(Update)>(&self, mut handler: F) {
        match block_on(self.inner.get_new_updates()) {
            Ok(updates) => {
                for update in updates {
                    self.inner.notify(&update);
                    let tx = lock(&self.updates_tx).clone();
                    if let Some(tx) = tx {
                        let _ = tx.send(update.clone());
                    }
                    handler(update);
                }
            }
            Err(e) => eprintln!("laser_tele: can't get updates: {e}"),
        }
    }
}

macro_rules! define_blocking_methods {
    ($( $(#[$attr:meta])* fn $name:ident($($arg:ident : $ty:ty),*) -> $ret:ty; )*) => {
        impl Bot {
            $(
                $(#[$attr])*
                pub fn $name(&self, $($arg: $ty),*) -> Result<$ret> {
                    block_on(self.inner.$name($($arg),*))
                }
            )*
        }
    };
}

macro_rules! define_blocking_functions {
    ($( $(#[$attr:meta])* fn $name:ident($($arg:ident : $ty:ty),*) -> $ret:ty; )*) => {
        $(
            $(#[$attr])*
            ///
            /// Works with the default bot, see [`init`]
            pub fn $name($($arg: $ty),*) -> Result<$ret> {
                default_bot()?.$name($($arg),*)
            }
        )*
    };
}

bot_methods!(define_blocking_methods);
bot_methods!(define_blocking_functions);

static DEFAULT_BOT: Mutex<Option<Bot>> = Mutex::new(None);

pub(crate) fn reset_default_bot() {
    *lock(&DEFAULT_BOT) = None;
}

/// Configures the default bot, the same as [`crate::init`]
pub fn init(config: Config) -> Result<()> {
    crate::init(config)
}

/// The default bot with the blocking API, it shares the state with [`crate::default_bot`]
pub fn default_bot() -> Result<Bot> {
    let mut default = lock(&DEFAULT_BOT);
    if default.is_none() {
        *default = Some(Bot::from_async(crate::default_bot()?));
    }
    Ok(default.clone().expect("the default bot is set"))
}

/// Requests updates of the default bot every `Config::timeout` and passes each of them to `Config::on_update`,
/// to the channel (if [`make_chan`] was called) and to `handler`. It returns only an error of creating the bot
pub fn run<F: FnMut(Update)>(handler: F) -> Result<()> {
    default_bot()?.run(handler);
    Ok(())
}

/// Requests new updates of the default bot once and passes each of them like [`run`]
pub fn update_request<F: FnMut(Update)>(handler: F) -> Result<()> {
    default_bot()?.update_request(handler);
    Ok(())
}

/// Creates the channel, to which updates of the default bot are sent, see [`Bot::make_chan`]
pub fn make_chan() -> Result<Receiver<Update>> {
    Ok(default_bot()?.make_chan())
}
