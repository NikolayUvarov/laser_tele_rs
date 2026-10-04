# laser_tele for Rust

A Telegram Bot API library with asynchronous (tokio) and blocking API.
It is the Rust version of the Go library [laser_tele](https://github.com/NikolayUvarov/laser_tele)
with the same functions, checked against Bot API 10.3.

Code Artisan: [https://github.com/LaserPes]

Visionary Director: [https://github.com/NikolayUvarov]

## Features

- All 27 kinds of updates of the Bot API: messages, buttons, polls, inline queries, payments, reactions, chat members, business messages...
- Messages with formatting, photos, videos, documents, inline keyboards; downloading files from users
- Polls and quizzes, inline mode, payments in Telegram Stars and through providers, games, reactions, join requests, business accounts
- Asynchronous and blocking API with the same methods; several bots in one program
- Errors with Telegram's codes, logs without texts of messages and with rotation; the token never gets to logs and errors
- Proxies: `Config::proxy` or `HTTPS_PROXY` (SOCKS5 with the `socks` feature)
- `call` for any other method of the Bot API

## Install

```toml
[dependencies]
laser_tele = { git = "https://github.com/NikolayUvarov/laser_tele_rs", tag = "v2.1.0" }
# for the asynchronous API
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Rust 1.88 or newer.

## Quick start, blocking

```rust,no_run
use laser_tele::{blocking, UpdateKind};

fn main() -> laser_tele::Result<()> {
    blocking::run(|update| match &update.kind {
        UpdateKind::Message(message) => {
            if let Err(e) = blocking::send_message(message.chat.id, &format!("You said: {}", message.text)) {
                eprintln!("Can't send message: {e}");
            }
        }
        UpdateKind::CallbackQuery(query) => {
            let _ = blocking::answer_callback_query(&query.id, &format!("Pressed {}", query.data));
        }
        _ => {}
    })
}
```

## Quick start, async

```rust,no_run
use laser_tele::UpdateKind;

#[tokio::main]
async fn main() -> laser_tele::Result<()> {
    laser_tele::run(|update| async move {
        if let UpdateKind::Message(message) = &update.kind {
            if let Err(e) = laser_tele::send_message(message.chat.id, &format!("You said: {}", message.text)).await {
                eprintln!("Can't send message: {e}");
            }
        }
    })
    .await
}
```

Create a bot with [@BotFather](https://t.me/BotFather) and run the program with its token:

```sh
TG_API_KEY=123456789:AAH... cargo run
```

The token is taken from `Config::api_key`, the `TG_API_KEY` environment variable or the `.APIKEY` file.

## Configuration

All fields of `Config` are optional:

```rust,no_run
use std::time::Duration;
use laser_tele::{Config, LogMode};

fn main() -> laser_tele::Result<()> {
    let config = Config {
        timeout: Duration::from_secs(2),         // interval between requests of updates, TIMEOUT env or 10 s
        allowed_updates: vec!["message".into(), "callback_query".into()], // all kinds by default
        log_mode: LogMode::WithoutContent,       // Full for debugging, Off for no logs
        log_dir: "logs".into(),                  // the current directory by default
        log_max_size: 1 << 20,                   // rotation, 10 MB by default
        download_dir: "files".into(),            // "downloadedFiles" by default
        proxy: "socks5://127.0.0.1:1080".into(), // HTTPS_PROXY by default
        ..Default::default()
    };
    laser_tele::init(config.on_update(|update| println!("Got {}", update.type_name())))?;
    // ...
    Ok(())
}
```

`api_url` points the bot to a [local Bot API server](https://github.com/tdlib/telegram-bot-api).

## Updates

`update.kind` is the kind of the update with its object, `update.type_name()` is its name in the Bot API
("message", "callback_query", "poll_answer"...), `update.raw` is the JSON received from Telegram:
any field not described in the library can be read from it. Optional objects are `Option`,
absent strings and numbers are empty: a message has a video if `message.video.is_some()`, text if `!message.text.is_empty()`.

The handler of `run` is called in the loop requesting updates. For slow jobs spawn a task (async) or a thread (blocking),
or read updates from the channel created by `make_chan` in several workers, see [examples/channel.rs](examples/channel.rs).

Some updates come only on conditions outside the code:

| Feature | Updates | Requirements |
|---|---|---|
| Polls | `Poll`, `PollAnswer` | votes come only for non-anonymous polls sent by the bot |
| Inline mode | `InlineQuery`, `ChosenInlineResult` | `/setinline` and `/setinlinefeedback` in @BotFather |
| Payments | `ShippingQuery`, `PreCheckoutQuery`, `Message::successful_payment` | `answer_pre_checkout_query` within 10 seconds |
| Games | `CallbackQuery` with `game_short_name` | `/newgame` in @BotFather |
| Reactions, members, boosts | `MessageReaction`, `ChatMember`, `ChatBoost`... | the bot is an administrator of the chat |
| Join requests | `ChatJoinRequest` | the bot is an administrator with the right to invite users |
| Business accounts | `BusinessConnection`, `BusinessMessage`... | the bot is connected to a business account |

## Errors

Every function sending a request returns `laser_tele::Result`:

```rust,no_run
use laser_tele::{blocking, Error};

fn greet(chat_id: i64) {
    match blocking::send_message(chat_id, "Hello") {
        Ok(sent) => println!("Sent message {}", sent.message_id),
        Err(Error::Api(e)) if e.error_code == 403 => println!("The user blocked the bot"),
        Err(Error::Api(e)) if e.retry_after > 0 => println!("Too many requests, wait {} seconds", e.retry_after),
        Err(Error::Api(e)) => println!("Telegram refused: {}", e.description),
        Err(e) => println!("Error: {e}"),
    }
}
```

## Logs

Every request and response is written to a log file named after the function: `sendMessage.log`,
`updateRequest.log`, `polls.log`, `payments.log`, `call.log`... Times are in UTC.

```text
2026/10/04 09:15:02 sendMessage   REQV: POST sendMessage chat_id=137511897
2026/10/04 09:15:02 sendMessage   RESP: 200 OK
2026/10/04 09:15:07 updateRequest   UPDATES: 794872551 message, 794872552 callback_query
```

## From Go to Rust

| Go | Rust, async | Rust, blocking |
|---|---|---|
| `DoLaserTeleInit(config)` | `laser_tele::init(config)?` | `blocking::init(config)?` |
| `LaserTeleRun(callback)` | `laser_tele::run(handler).await?` | `blocking::run(handler)?` |
| `UpdateRequest(callback)` | `laser_tele::update_request(handler).await?` | `blocking::update_request(handler)?` |
| `MakeChan()`, `TgChan` | `let rx = laser_tele::make_chan()?` | `let rx = blocking::make_chan()?` |
| `NewBot(config)` | `Bot::new(config)?` | `blocking::Bot::new(config)?` |
| `SendMessage(chatID, text)` | `send_message(chat_id, text).await?` | `blocking::send_message(chat_id, text)?` |
| `LaserTeleConfigT{APIKEY, Timeout, CallbackOnUpdate}` | `Config { api_key, timeout, ..Default::default() }.on_update(f)` | the same |
| `update.Type()`, `update.UpdateMessage` | `update.type_name()`, `UpdateKind::Message(message)`, `update.message()` | the same |
| `AddButton(text, data)`, `Button{Text, URL}` | `add_button(text, data)`, `Button::callback`, `Button::url` | the same |
| `NewInlineArticle(id, title, text)` | `InlineQueryResult::article(id, title, text)` | the same |
| `GameMessage{ChatID, MessageID}` | `GameMessage::Chat { chat_id, message_id }` | the same |
| `*APIError` | `Error::Api(ApiError)`, `err.api()` | the same |
| `LogWithoutContent`, `LogFull`, `LogOff` | `LogMode::WithoutContent`, `Full`, `Off` | the same |
| `LoadFile(chatID, fileID)` | `load_file(file_id)` | the same |

Every other function has the same name in snake case: `SendPoll` is `send_poll`, `AnswerPreCheckoutQuery` is
`answer_pre_checkout_query`... Differences of the Rust version:

- `init` and `run` return an error instead of stopping the program when the token is not found;
- functions sending messages return the sent `Message`;
- the library doesn't print to the standard output, only errors of requesting updates to the standard error;
- additions: `Config::proxy`, `Config::api_url`, `Message::command()`, `Message::entity_text()`.

## Examples

Complete bots in [examples/](examples), run them with `TG_API_KEY=... cargo run --example <name>`:

| Example | API | What it shows |
|---|---|---|
| [echo](examples/echo.rs) | blocking | the smallest bot |
| [echo_async](examples/echo_async.rs) | async | the smallest bot |
| [keyboard](examples/keyboard.rs) | blocking | buttons, answers to buttons, editing messages |
| [files](examples/files.rs) | blocking | downloading files from users, sending documents |
| [polls](examples/polls.rs) | async | polls, quizzes, votes, results |
| [inline](examples/inline.rs) | async | inline mode |
| [payments](examples/payments.rs) | blocking | a shop selling for Telegram Stars, refunds |
| [game](examples/game.rs) | async | an HTML5 game with a server for scores |
| [groups](examples/groups.rs) | async | join requests, greetings, reactions, boosts |
| [business](examples/business.rs) | blocking | answering on behalf of a business account |
| [several_bots](examples/several_bots.rs) | async | two bots in one program |
| [channel](examples/channel.rs) | blocking | processing updates in several threads |

The API reference: `cargo doc --open`.

## Development

```sh
cargo test                      # tests against a fake Telegram server
cargo clippy --all-targets
cargo doc --no-deps
```

## License

[Apache License 2.0](LICENSE)
