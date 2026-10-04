use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::logger::{self, Logger};
use crate::*;

const DEFAULT_API_URL: &str = "https://api.telegram.org";
const DEFAULT_DOWNLOAD_DIR: &str = "downloadedFiles";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
// timeouts of requests, so a hung connection can't stall the bot forever
const API_TIMEOUT: Duration = Duration::from_secs(60);
const FILE_TIMEOUT: Duration = Duration::from_secs(600);

/// A Telegram bot with the asynchronous API. Several bots with different tokens can work in one program.
///
/// `Bot` is cheap to clone: clones share the connection pool and the state of the bot.
/// For the blocking API see [`blocking::Bot`](crate::blocking::Bot).
///
/// ```no_run
/// # async fn example() -> laser_tele::Result<()> {
/// let bot = laser_tele::Bot::new(laser_tele::Config::default())?;
/// let handler_bot = bot.clone();
/// bot.run(move |update| {
///     let bot = handler_bot.clone();
///     async move {
///         if let Some(message) = update.message() {
///             if let Err(e) = bot.send_message(message.chat.id, &message.text).await {
///                 eprintln!("Can't send message: {e}");
///             }
///         }
///     }
/// })
/// .await;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Bot {
    pub(crate) inner: Arc<Inner>,
}

pub(crate) struct Inner {
    timeout: Duration,
    api_link: String,
    file_link: String,
    download_dir: PathBuf,
    on_update: Option<UpdateCallback>,
    allowed_updates: Vec<String>,
    logger: Logger,
    client: reqwest::Client,
    file_client: reqwest::Client,
    poll: Mutex<PollState>,
    updates_tx: Mutex<Option<mpsc::Sender<Update>>>,
}

#[derive(Default)]
struct PollState {
    initialized: bool,
    last_update_id: i64,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ApiResponse {
    ok: bool,
    error_code: i64,
    description: String,
    parameters: ResponseParameters,
    result: Value,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ResponseParameters {
    migrate_to_chat_id: i64,
    retry_after: i64,
}

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn api_error(method: &str, error_code: i64, description: impl Into<String>) -> Error {
    Error::Api(ApiError {
        method: method.to_string(),
        error_code,
        description: description.into(),
        retry_after: 0,
        migrate_to_chat_id: 0,
    })
}

fn build_client(proxy: &str, timeout: Duration) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder().timeout(timeout);
    if !proxy.is_empty() {
        let proxy = reqwest::Proxy::all(proxy).map_err(|e| {
            Error::Config(format!(
                "proxy {}: {}",
                logger::hide_password(proxy),
                e.without_url()
            ))
        })?;
        builder = builder.proxy(proxy);
    }
    builder
        .build()
        .map_err(|e| Error::Config(e.without_url().to_string()))
}

impl Bot {
    /// Creates a bot. If `config.api_key` is empty, the token is taken from the `TG_API_KEY` environment
    /// variable or the `.APIKEY` file; if `config.timeout` is zero, it is taken from the `TIMEOUT`
    /// environment variable (seconds, 10 by default)
    pub fn new(config: Config) -> Result<Bot> {
        let mut api_key = config.api_key.trim().to_string();
        if api_key.is_empty() {
            api_key = std::env::var("TG_API_KEY")
                .unwrap_or_default()
                .trim()
                .to_string();
        }
        if api_key.is_empty() {
            api_key = std::fs::read_to_string(".APIKEY")
                .map(|key| key.trim().to_string())
                .unwrap_or_default();
        }
        if api_key.is_empty() {
            return Err(Error::NoToken);
        }

        let timeout = if !config.timeout.is_zero() {
            config.timeout
        } else {
            match std::env::var("TIMEOUT") {
                Ok(value) if !value.is_empty() => match value.trim().parse::<u64>() {
                    Ok(seconds) if seconds > 0 => Duration::from_secs(seconds),
                    _ => {
                        eprintln!("laser_tele: wrong TIMEOUT value, using 10 seconds");
                        DEFAULT_TIMEOUT
                    }
                },
                _ => DEFAULT_TIMEOUT,
            }
        };

        let logger = Logger::new(&config, &api_key)?;
        let download_dir = if config.download_dir.as_os_str().is_empty() {
            PathBuf::from(DEFAULT_DOWNLOAD_DIR)
        } else {
            config.download_dir.clone()
        };
        let allowed_updates = if config.allowed_updates.is_empty() {
            ALL_UPDATE_TYPES
                .iter()
                .map(|kind| kind.to_string())
                .collect()
        } else {
            config.allowed_updates.clone()
        };
        let api_url = if config.api_url.is_empty() {
            DEFAULT_API_URL
        } else {
            config.api_url.trim_end_matches('/')
        };

        Ok(Bot {
            inner: Arc::new(Inner {
                api_link: format!("{api_url}/bot{api_key}"),
                file_link: format!("{api_url}/file/bot{api_key}"),
                timeout,
                download_dir,
                on_update: config.on_update.clone(),
                allowed_updates,
                logger,
                client: build_client(&config.proxy, API_TIMEOUT)?,
                file_client: build_client(&config.proxy, FILE_TIMEOUT)?,
                poll: Mutex::new(PollState::default()),
                updates_tx: Mutex::new(None),
            }),
        })
    }

    /// Interval between requests of updates
    pub fn timeout(&self) -> Duration {
        self.inner.timeout
    }

    /// Creates the channel, to which updates are sent, and returns its receiver. It must be called before `run`.
    /// The channel holds one update: until it is read, processing of the next update waits
    pub fn make_chan(&self) -> mpsc::Receiver<Update> {
        let (tx, rx) = mpsc::channel(1);
        *lock(&self.inner.updates_tx) = Some(tx);
        rx
    }

    /// Requests updates every `timeout` and passes each of them to `Config::on_update`,
    /// to the channel (if `make_chan` was called) and to `handler`. It never returns.
    ///
    /// Updates sent before the start of the bot are skipped, so the bot doesn't answer old messages.
    /// The handler is awaited before the next update: spawn a task for slow jobs
    pub async fn run<F, Fut>(&self, mut handler: F)
    where
        F: FnMut(Update) -> Fut,
        Fut: Future<Output = ()>,
    {
        loop {
            self.update_request(&mut handler).await;
            tokio::time::sleep(self.inner.timeout).await;
        }
    }

    /// Requests new updates once and passes each of them to `Config::on_update`,
    /// to the channel (if `make_chan` was called) and to `handler`
    pub async fn update_request<F, Fut>(&self, mut handler: F)
    where
        F: FnMut(Update) -> Fut,
        Fut: Future<Output = ()>,
    {
        match self.get_new_updates().await {
            Ok(updates) => {
                for update in updates {
                    self.notify(&update);
                    let tx = lock(&self.inner.updates_tx).clone();
                    if let Some(tx) = tx {
                        let _ = tx.send(update.clone()).await;
                    }
                    handler(update).await;
                }
            }
            Err(e) => eprintln!("laser_tele: can't get updates: {e}"),
        }
    }

    /// Requests new updates and confirms the previous ones. Updates sent before the start are skipped
    pub(crate) async fn get_new_updates(&self) -> Result<Vec<Update>> {
        let offset = {
            let state = lock(&self.inner.poll);
            if !state.initialized {
                // on the first request only the last update is requested, older ones are dropped by Telegram
                Some(-1)
            } else if state.last_update_id != 0 {
                // confirm processed updates, so Telegram doesn't send them again
                Some(state.last_update_id + 1)
            } else {
                None
            }
        };
        let mut params = json!({ "allowed_updates": self.inner.allowed_updates });
        if let Some(offset) = offset {
            params["offset"] = json!(offset);
        }

        let result = self
            .call_json("updateRequest", "getUpdates", &params)
            .await?;
        let updates: Vec<Update> = if result.is_null() {
            Vec::new()
        } else {
            serde_json::from_value(result)?
        };

        {
            let mut state = lock(&self.inner.poll);
            if !state.initialized {
                // updates sent before the start of the bot are skipped
                state.initialized = true;
                if let Some(last) = updates.last() {
                    state.last_update_id = last.update_id;
                }
                return Ok(Vec::new());
            }
            if let Some(last) = updates.last() {
                state.last_update_id = last.update_id;
            }
        }

        if !updates.is_empty() && self.inner.logger.mode != LogMode::Full {
            let ids: Vec<String> = updates
                .iter()
                .map(|update| format!("{} {}", update.update_id, update.type_name()))
                .collect();
            self.log("updateRequest", &format!("UPDATES: {}", ids.join(", ")));
        }
        Ok(updates)
    }

    /// Calls `Config::on_update`
    pub(crate) fn notify(&self, update: &Update) {
        if let Some(callback) = &self.inner.on_update {
            callback(update);
        }
    }

    /// Calls any Bot API method (<https://core.telegram.org/bots/api#available-methods>) with `params`
    /// (a JSON object) and returns the `result` field of the response.
    /// Use it for methods that have no own function in the library
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let params = if params.is_null() { json!({}) } else { params };
        self.call_json("call", method, &params).await
    }

    pub(crate) fn log(&self, name: &str, line: &str) {
        self.inner.logger.write(name, line);
    }

    fn network_error(&self, log_name: &str, e: reqwest::Error) -> Error {
        let e = e.without_url();
        self.log(log_name, &format!("RESP: CONNECTION_ERROR {e}"));
        Error::Network(e)
    }

    /// Sends params as JSON to the Bot API method and returns the `result` field of the response.
    /// The request and the response are written to the log `log_name`
    pub(crate) async fn call_json(
        &self,
        log_name: &str,
        method: &str,
        params: &Value,
    ) -> Result<Value> {
        let link = format!("{}/{}", self.inner.api_link, method);
        if self.inner.logger.mode == LogMode::Full {
            self.log(log_name, &format!("REQV: POST {link} {params}"));
        } else {
            self.log(
                log_name,
                &format!(
                    "REQV: POST {method} {}",
                    logger::params_without_content(params)
                ),
            );
        }

        let response = self
            .inner
            .client
            .post(&link)
            .json(params)
            .send()
            .await
            .map_err(|e| self.network_error(log_name, e))?;
        self.read_response(log_name, method, response).await
    }

    /// Calls the Bot API method and converts the result of the response to `T`
    pub(crate) async fn call_into<T: DeserializeOwned>(
        &self,
        log_name: &str,
        method: &str,
        params: &Value,
    ) -> Result<T> {
        let result = self.call_json(log_name, method, params).await?;
        serde_json::from_value(result)
            .map_err(|e| api_error(method, 0, format!("wrong result: {e}")))
    }

    /// Reads the response of the Bot API method, returns `Error::Api` if Telegram refused the request
    async fn read_response(
        &self,
        log_name: &str,
        method: &str,
        response: reqwest::Response,
    ) -> Result<Value> {
        let status = response.status().as_u16();
        let body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                let e = e.without_url();
                self.log(log_name, &format!("RESP: READ_ERROR {e}"));
                return Err(Error::Network(e));
            }
        };

        let parsed: std::result::Result<ApiResponse, _> = serde_json::from_str(&body);
        let line = match &parsed {
            _ if self.inner.logger.mode == LogMode::Full => format!("RESP: {status} {body}"),
            Err(_) => format!("RESP: {status} WRONG_RESPONSE"),
            Ok(response) if !response.ok => format!("RESP: {status} {}", response.description),
            Ok(_) => format!("RESP: {status} OK"),
        };
        self.log(log_name, &line);

        let response = parsed
            .map_err(|e| api_error(method, i64::from(status), format!("wrong response: {e}")))?;
        if !response.ok {
            return Err(Error::Api(ApiError {
                method: method.to_string(),
                error_code: response.error_code,
                description: response.description,
                retry_after: response.parameters.retry_after,
                migrate_to_chat_id: response.parameters.migrate_to_chat_id,
            }));
        }
        Ok(response.result)
    }

    /// Uploads the local file to the chat with the Bot API method (sendPhoto, sendVideo...).
    /// `field` is the name of the form field for the file (photo, video...)
    async fn send_file(
        &self,
        log_name: &str,
        method: &str,
        field: &str,
        chat_id: i64,
        caption: &str,
        path: &Path,
    ) -> Result<Message> {
        let data = match tokio::fs::read(path).await {
            Ok(data) => data,
            Err(e) => {
                self.log(log_name, &format!("REQV: CANT_OPEN_FILE {e}"));
                return Err(e.into());
            }
        };
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| field.to_string());
        let mut form = reqwest::multipart::Form::new().text("chat_id", chat_id.to_string());
        if !caption.is_empty() {
            form = form.text("caption", caption.to_string());
        }
        form = form.part(
            field.to_string(),
            reqwest::multipart::Part::bytes(data).file_name(file_name),
        );

        let link = format!("{}/{}", self.inner.api_link, method);
        if self.inner.logger.mode == LogMode::Full {
            self.log(
                log_name,
                &format!(
                    "REQV: POST {link} chat_id={chat_id} {field}={} caption={caption:?}",
                    path.display()
                ),
            );
        } else {
            self.log(
                log_name,
                &format!(
                    "REQV: POST {method} chat_id={chat_id} {field}={}",
                    path.display()
                ),
            );
        }

        let response = self
            .inner
            .file_client
            .post(&link)
            .multipart(form)
            .send()
            .await
            .map_err(|e| self.network_error(log_name, e))?;
        let result = self.read_response(log_name, method, response).await?;
        serde_json::from_value(result)
            .map_err(|e| api_error(method, 0, format!("wrong result: {e}")))
    }

    /// Uploads a local photo file to the chat, `caption` is the text under it (up to 1024 characters)
    pub async fn send_photo(
        &self,
        chat_id: i64,
        caption: &str,
        path: impl AsRef<Path> + Send,
    ) -> Result<Message> {
        self.send_file(
            "sendPhoto",
            "sendPhoto",
            "photo",
            chat_id,
            caption,
            path.as_ref(),
        )
        .await
    }

    /// Uploads a local video file to the chat, `caption` is the text under it
    pub async fn send_video(
        &self,
        chat_id: i64,
        caption: &str,
        path: impl AsRef<Path> + Send,
    ) -> Result<Message> {
        self.send_file(
            "sendVideo",
            "sendVideo",
            "video",
            chat_id,
            caption,
            path.as_ref(),
        )
        .await
    }

    /// Uploads a local file to the chat as a document, `caption` is the text under it
    pub async fn send_document(
        &self,
        chat_id: i64,
        caption: &str,
        path: impl AsRef<Path> + Send,
    ) -> Result<Message> {
        self.send_file(
            "sendDocument",
            "sendDocument",
            "document",
            chat_id,
            caption,
            path.as_ref(),
        )
        .await
    }

    /// Downloads a file sent by a user (its `file_id`) to `Config::download_dir` and returns its path.
    /// Bots can download files up to 20 MB
    pub async fn load_file(&self, file_id: &str) -> Result<PathBuf> {
        let file: File = self
            .call_into("loadFile", "getFile", &json!({ "file_id": file_id }))
            .await?;
        if file.file_path.is_empty() {
            return Err(api_error("getFile", 0, "no file_path in response"));
        }
        let link = format!("{}/{}", self.inner.file_link, file.file_path);
        self.download(&link, &file.file_path).await
    }

    /// Downloads the file by `link` and saves it to `file_path` in `Config::download_dir`
    pub async fn file_download(&self, link: &str, file_path: &str) -> Result<PathBuf> {
        self.download(link, file_path).await
    }

    async fn download(&self, link: &str, file_path: &str) -> Result<PathBuf> {
        let response = self
            .inner
            .file_client
            .get(link)
            .send()
            .await
            .map_err(|e| self.network_error("fileLoad", e))?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();
        self.log(
            "fileLoad",
            &format!(
                "Response status: {status}, Content-Type: {content_type:>12}, Loading URL: {link}"
            ),
        );
        if !status.is_success() {
            return Err(api_error(
                "downloadFile",
                i64::from(status.as_u16()),
                status.canonical_reason().unwrap_or(""),
            ));
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| self.network_error("fileLoad", e))?;
        let path = self.inner.download_dir.join(file_path);
        let saved = async {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&path, &data).await
        };
        if let Err(e) = saved.await {
            self.log("fileLoad", &format!("CANT_SAVE_FILE {e}"));
            return Err(e.into());
        }
        Ok(path)
    }
}
