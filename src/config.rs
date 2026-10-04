use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::Update;

/// A function called for every new update, see [`Config::on_update`]
pub type UpdateCallback = Arc<dyn Fn(&Update) + Send + Sync>;

/// What is written to the log files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogMode {
    /// Requests and responses without texts of messages: methods, chat IDs, statuses, errors
    #[default]
    WithoutContent,
    /// Full requests and responses with texts of messages, for debugging
    Full,
    /// No logs
    Off,
}

/// Configuration of a bot for [`Bot::new`](crate::Bot::new) and [`init`](crate::init). All fields are optional:
///
/// ```no_run
/// use std::time::Duration;
///
/// let config = laser_tele::Config {
///     timeout: Duration::from_secs(2),
///     log_dir: "logs".into(),
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Default)]
pub struct Config {
    /// The token of the bot; if empty, it is taken from the `TG_API_KEY` environment variable or the `.APIKEY` file
    pub api_key: String,
    /// Interval between requests of updates; if zero, it is taken from the `TIMEOUT`
    /// environment variable (seconds), 10 seconds by default
    pub timeout: Duration,
    /// Called for every new update, in addition to the handler passed to `run`
    pub on_update: Option<UpdateCallback>,
    /// Kinds of updates the bot receives, [`ALL_UPDATE_TYPES`](crate::ALL_UPDATE_TYPES) by default.
    /// Telegram sends some of them only if the bot is an administrator in the chat
    pub allowed_updates: Vec<String>,
    /// What is written to the log files
    pub log_mode: LogMode,
    /// Directory of the log files, the current directory by default
    pub log_dir: PathBuf,
    /// Max size of a log file in bytes (10 MB by default). The larger file is renamed
    /// to `<name>.log.1`, the previous `<name>.log.1` is deleted
    pub log_max_size: u64,
    /// Directory for files downloaded by `load_file`, `downloadedFiles` by default
    pub download_dir: PathBuf,
    /// Proxy for requests to Telegram: `http://user:password@host:port` or `socks5://host:port`
    /// (with the `socks` feature). If empty, `HTTPS_PROXY` and `NO_PROXY` environment variables are used
    pub proxy: String,
    /// Address of the Bot API server, `https://api.telegram.org` by default.
    /// Set it for a [local Bot API server](https://github.com/tdlib/telegram-bot-api)
    pub api_url: String,
}

impl Config {
    /// Sets the function called for every new update
    pub fn on_update(mut self, callback: impl Fn(&Update) + Send + Sync + 'static) -> Self {
        self.on_update = Some(Arc::new(callback));
        self
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field(
                "api_key",
                &if self.api_key.is_empty() {
                    ""
                } else {
                    "<APIKEY>"
                },
            )
            .field("timeout", &self.timeout)
            .field("on_update", &self.on_update.as_ref().map(|_| "callback"))
            .field("allowed_updates", &self.allowed_updates)
            .field("log_mode", &self.log_mode)
            .field("log_dir", &self.log_dir)
            .field("log_max_size", &self.log_max_size)
            .field("download_dir", &self.download_dir)
            .field("proxy", &crate::logger::hide_password(&self.proxy))
            .field("api_url", &self.api_url)
            .finish()
    }
}
