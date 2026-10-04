use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Config, LogMode, Result};

const DEFAULT_LOG_MAX_SIZE: u64 = 10 << 20;

// all bots write logs under the mutex, so rotation of a log file is safe even if bots share it
static LOG_MUTEX: Mutex<()> = Mutex::new(());

// params without texts of messages, they are logged in LogMode::WithoutContent
const PARAMS_TO_LOG: &[&str] = &[
    "chat_id",
    "message_id",
    "inline_message_id",
    "user_id",
    "offset",
    "file_id",
    "ok",
    "callback_query_id",
    "inline_query_id",
    "guest_query_id",
    "shipping_query_id",
    "pre_checkout_query_id",
    "business_connection_id",
    "game_short_name",
];

pub(crate) struct Logger {
    pub(crate) mode: LogMode,
    dir: PathBuf,
    max_size: u64,
    api_key: String,
}

impl Logger {
    pub(crate) fn new(config: &Config, api_key: &str) -> Result<Logger> {
        let logger = Logger {
            mode: config.log_mode,
            dir: config.log_dir.clone(),
            max_size: if config.log_max_size == 0 {
                DEFAULT_LOG_MAX_SIZE
            } else {
                config.log_max_size
            },
            api_key: api_key.to_string(),
        };
        if !logger.dir.as_os_str().is_empty() && logger.mode != LogMode::Off {
            fs::create_dir_all(&logger.dir)?;
        }
        Ok(logger)
    }

    /// Makes a new record in the file `<name>.log`.
    /// If the file is larger than max_size, it is renamed to `<name>.log.1` (the previous one is deleted)
    pub(crate) fn write(&self, name: &str, line: &str) {
        if self.mode == LogMode::Off {
            return;
        }
        let _guard = LOG_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let file_name = self.dir.join(format!("{name}.log"));
        if let Ok(metadata) = fs::metadata(&file_name)
            && metadata.len() >= self.max_size
        {
            let rotated = self.dir.join(format!("{name}.log.1"));
            let _ = fs::remove_file(&rotated);
            if let Err(e) = fs::rename(&file_name, &rotated) {
                eprintln!("laser_tele: can't rotate log file: {e}");
            }
        }

        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o640);
        match options.open(&file_name) {
            Ok(mut file) => {
                let _ = writeln!(
                    file,
                    "{} {}   {}",
                    timestamp(),
                    name,
                    hide_key(line, &self.api_key)
                );
            }
            Err(e) => eprintln!("laser_tele: can't open log file: {e}"),
        }
    }
}

/// Replaces the token in s, so it doesn't get to logs and errors
pub(crate) fn hide_key(s: &str, api_key: &str) -> String {
    if api_key.is_empty() {
        return s.to_string();
    }
    s.replace(api_key, "<APIKEY>")
}

/// Hides the password in a URL like `http://user:password@host`
pub(crate) fn hide_password(url: &str) -> String {
    match (url.find("://"), url.rfind('@')) {
        (Some(scheme), Some(at)) if at > scheme => {
            let credentials = &url[scheme + 3..at];
            match credentials.find(':') {
                Some(colon) => format!(
                    "{}{}:***{}",
                    &url[..scheme + 3],
                    &credentials[..colon],
                    &url[at..]
                ),
                None => url.to_string(),
            }
        }
        _ => url.to_string(),
    }
}

/// Leaves only params without texts of messages, for LogMode::WithoutContent
pub(crate) fn params_without_content(params: &serde_json::Value) -> String {
    let Some(object) = params.as_object() else {
        return String::new();
    };
    let mut names: Vec<&String> = object
        .keys()
        .filter(|name| PARAMS_TO_LOG.contains(&name.as_str()))
        .collect();
    names.sort();
    names
        .iter()
        .map(|name| match &object[name.as_str()] {
            serde_json::Value::String(s) => format!("{name}={s}"),
            value => format!("{name}={value}"),
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Current UTC time like "2026/10/04 09:15:02"
fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (days, rest) = (seconds.div_euclid(86400), seconds.rem_euclid(86400));
    // civil date from days since 1970-01-01, http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}/{month:02}/{day:02} {:02}:{:02}:{:02}",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_secrets() {
        assert_eq!(
            hide_key("https://x/bot123:ABC/getMe", "123:ABC"),
            "https://x/bot<APIKEY>/getMe"
        );
        assert_eq!(
            hide_password("http://user:secret@proxy:8080"),
            "http://user:***@proxy:8080"
        );
        assert_eq!(hide_password("socks5://proxy:1080"), "socks5://proxy:1080");
    }

    #[test]
    fn logs_only_params_without_content() {
        let params = serde_json::json!({"chat_id": 137, "text": "secret", "callback_query_id": "q1", "ok": true});
        assert_eq!(
            params_without_content(&params),
            "callback_query_id=q1&chat_id=137&ok=true"
        );
    }

    #[test]
    fn formats_timestamp() {
        let ts = timestamp();
        assert_eq!(ts.len(), 19, "{ts}");
        assert!(ts.starts_with("20"), "{ts}");
    }
}
