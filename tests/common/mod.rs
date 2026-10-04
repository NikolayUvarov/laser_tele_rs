//! A fake Telegram Bot API server for tests.
//!
//! getUpdates returns queued updates and confirms them by offset, getFile returns the path "files/<file_id>",
//! files are downloaded from `files`, other methods get the response set by `set_response`

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{FromRequest, Multipart, Request, State};
use axum::http::{StatusCode, header};
use axum::response::Response;
use laser_tele::Config;
use serde_json::{Value, json};

pub const TEST_KEY: &str = "123456:TEST-KEY";

/// A request received by the fake server
#[derive(Debug, Clone, Default)]
pub struct Recorded {
    pub method: String,
    /// Params sent as JSON
    pub json: Value,
    /// Text fields of a multipart form
    pub form: HashMap<String, String>,
    pub file_field: String,
    pub file_name: String,
    pub file_content: Vec<u8>,
}

struct FakeState {
    api_key: String,
    updates: Vec<Value>,
    requests: Vec<Recorded>,
    status: u16,
    body: String,
    files: HashMap<String, Vec<u8>>,
}

type Shared = Arc<Mutex<FakeState>>;

pub struct FakeTelegram {
    pub url: String,
    state: Shared,
}

impl FakeTelegram {
    /// Starts the server in its own thread, so it serves both async and blocking tests
    pub fn start(api_key: &str) -> FakeTelegram {
        let state = Arc::new(Mutex::new(FakeState {
            api_key: api_key.to_string(),
            updates: Vec::new(),
            requests: Vec::new(),
            status: 200,
            body: r#"{"ok":true,"result":{}}"#.to_string(),
            files: HashMap::new(),
        }));
        let (tx, rx) = std::sync::mpsc::channel();
        let server_state = state.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                tx.send(listener.local_addr().unwrap()).unwrap();
                let app = Router::new().fallback(handle).with_state(server_state);
                axum::serve(listener, app).await.unwrap();
            });
        });
        let address = rx.recv_timeout(Duration::from_secs(10)).unwrap();
        FakeTelegram {
            url: format!("http://{address}"),
            state,
        }
    }

    pub fn add_updates(&self, updates: Vec<Value>) {
        self.state.lock().unwrap().updates.extend(updates);
    }

    pub fn set_response(&self, status: u16, body: &str) {
        let mut state = self.state.lock().unwrap();
        state.status = status;
        state.body = body.to_string();
    }

    pub fn add_file(&self, path: &str, content: &str) {
        self.state
            .lock()
            .unwrap()
            .files
            .insert(path.to_string(), content.as_bytes().to_vec());
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.state.lock().unwrap().requests.clone()
    }

    pub fn last_request(&self) -> Recorded {
        self.requests().pop().expect("no requests")
    }
}

fn json_response(status: u16, body: String) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}

async fn handle(State(state): State<Shared>, request: Request) -> Response {
    let path = request.uri().path().to_string();
    let api_key = state.lock().unwrap().api_key.clone();

    if let Some(file_path) = path.strip_prefix(&format!("/file/bot{api_key}/")) {
        return match state.lock().unwrap().files.get(file_path) {
            Some(content) => Response::new(Body::from(content.clone())),
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap(),
        };
    }
    let Some(method) = path.strip_prefix(&format!("/bot{api_key}/")) else {
        return json_response(
            401,
            r#"{"ok":false,"error_code":401,"description":"Unauthorized"}"#.into(),
        );
    };

    let mut recorded = Recorded {
        method: method.to_string(),
        ..Default::default()
    };
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    if content_type.starts_with("multipart/form-data") {
        let mut multipart = Multipart::from_request(request, &()).await.unwrap();
        while let Some(field) = multipart.next_field().await.unwrap() {
            let name = field.name().unwrap_or("").to_string();
            match field.file_name().map(str::to_string) {
                Some(file_name) => {
                    recorded.file_field = name;
                    recorded.file_name = file_name;
                    recorded.file_content = field.bytes().await.unwrap().to_vec();
                }
                None => {
                    recorded.form.insert(name, field.text().await.unwrap());
                }
            }
        }
    } else {
        let body = axum::body::to_bytes(request.into_body(), usize::MAX)
            .await
            .unwrap();
        recorded.json = serde_json::from_slice(&body).unwrap_or(Value::Null);
    }

    let mut state = state.lock().unwrap();
    state.requests.push(recorded.clone());
    match recorded.method.as_str() {
        "getUpdates" => {
            if let Some(offset) = recorded.json.get("offset").and_then(Value::as_i64) {
                let count = state.updates.len() as i64;
                let updates = std::mem::take(&mut state.updates);
                state.updates = updates
                    .into_iter()
                    .enumerate()
                    .filter(|(num, update)| {
                        let id = update["update_id"].as_i64().unwrap();
                        if offset < 0 {
                            *num as i64 >= count + offset
                        } else {
                            id >= offset
                        }
                    })
                    .map(|(_, update)| update)
                    .collect();
            }
            json_response(
                200,
                json!({ "ok": true, "result": state.updates }).to_string(),
            )
        }
        "getFile" => {
            let file_id = recorded.json["file_id"].as_str().unwrap_or("").to_string();
            if file_id == "wrong" {
                return json_response(
                    400,
                    r#"{"ok":false,"error_code":400,"description":"Bad Request: invalid file_id"}"#
                        .into(),
                );
            }
            json_response(
                200,
                json!({ "ok": true, "result": { "file_id": file_id, "file_path": format!("files/{file_id}") } }).to_string(),
            )
        }
        _ => json_response(state.status, state.body.clone()),
    }
}

/// A bot connected to a new fake server, its logs and downloads are in a temporary directory
pub struct TestBot {
    pub bot: laser_tele::Bot,
    pub tg: FakeTelegram,
    pub dir: tempfile::TempDir,
}

impl TestBot {
    pub fn log_dir(&self) -> PathBuf {
        self.dir.path().join("logs")
    }

    pub fn download_dir(&self) -> PathBuf {
        self.dir.path().join("downloads")
    }

    pub fn read_log(&self, name: &str) -> String {
        std::fs::read_to_string(self.log_dir().join(format!("{name}.log"))).unwrap_or_default()
    }
}

pub fn test_config(
    tg: &FakeTelegram,
    dir: &tempfile::TempDir,
    api_key: &str,
    config: Config,
) -> Config {
    Config {
        api_key: api_key.to_string(),
        api_url: tg.url.clone(),
        timeout: Duration::from_millis(10),
        log_dir: dir.path().join("logs"),
        download_dir: dir.path().join("downloads"),
        ..config
    }
}

pub fn new_bot(config: Config) -> TestBot {
    let api_key = if config.api_key.is_empty() {
        TEST_KEY.to_string()
    } else {
        config.api_key.clone()
    };
    let dir = tempfile::tempdir().unwrap();
    let tg = FakeTelegram::start(&api_key);
    let bot = laser_tele::Bot::new(test_config(&tg, &dir, &api_key, config)).unwrap();
    TestBot { bot, tg, dir }
}

pub fn test_user() -> Value {
    json!({ "id": 137511897, "is_bot": false, "first_name": "Nikolos", "username": "nikolosu", "language_code": "ru" })
}

pub fn test_chat() -> Value {
    json!({ "id": 137511897, "first_name": "Nikolos", "username": "nikolosu", "type": "private" })
}

pub fn test_message(message_id: i64, text: &str) -> Value {
    json!({ "message_id": message_id, "from": test_user(), "chat": test_chat(), "date": 1692347648, "text": text })
}

pub fn test_update(update_id: i64, kind: &str, object: Value) -> Value {
    json!({ "update_id": update_id, kind: object })
}

pub fn text_update(update_id: i64, text: &str) -> Value {
    test_update(
        update_id,
        "message",
        test_message(update_id % 1000 + 1, text),
    )
}
