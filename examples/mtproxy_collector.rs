//! The bot collecting MTProxy servers for the registry of the Bot API server
//! <https://github.com/NikolayUvarov/telegram-bot-api>. It takes links from posts of channels, where it is an
//! administrator, and from messages of its administrators, including posts forwarded from any channel: from texts,
//! captions, hidden links and URL buttons ("Connect"). The server checks the servers and switches its bots to a
//! working one.
//!
//! The server must be started with `--mtproxy-admins=<identifier of this bot>` (the number before ':' in the token),
//! and the bot must connect to Telegram through the server once (e.g. with `--mtproxy`) before it can manage
//! the registry.
//!
//! Commands of administrators: /proxies (the registry), /check, /use <id>, /remove <id>.
//!
//! Run: TG_API_KEY=<token> BOT_API_URL=http://localhost:8081 MTPROXY_ADMINS=<your user id> \
//! MTPROXY_CHANNELS=<channel ids or @usernames, all channels if empty> cargo run --example mtproxy_collector

use std::collections::HashSet;
use std::time::Duration;

use laser_tele::{Chat, Config, Message, UpdateKind, Value, blocking, json};
use serde::Deserialize;

/// Keeps /proxies within the limit of a message
const MAX_LIST_LENGTH: usize = 3500;

/// An MTProxy server from getMTProxies
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RegistryProxy {
    id: i64,
    server: String,
    port: i64,
    secret_type: String,
    domain: String,
    /// "unchecked", "working" or "failing"
    state: String,
    is_active: bool,
    last_error: String,
    ping: f64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Registry {
    bot_count: i64,
    connected_bot_count: i64,
    is_checking: bool,
    proxies: Vec<RegistryProxy>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AddResult {
    added: i64,
    known: i64,
    errors: Vec<AddError>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AddError {
    link: String,
    error: String,
}

struct Collector {
    bot: blocking::Bot,
    /// Identifiers and @usernames of channels, all channels if empty
    channels: HashSet<String>,
    /// Users, who can add servers and use commands
    admins: HashSet<i64>,
}

impl Collector {
    fn new(bot: blocking::Bot, channels: &str, admins: &str) -> Result<Collector, String> {
        let channels = channels
            .split(',')
            .map(|channel| channel.trim().to_lowercase())
            .filter(|channel| !channel.is_empty())
            .collect();
        let mut admin_ids = HashSet::new();
        for admin in admins
            .split(',')
            .map(str::trim)
            .filter(|admin| !admin.is_empty())
        {
            let id = admin
                .parse()
                .map_err(|_| format!("wrong user identifier {admin:?} in MTPROXY_ADMINS"))?;
            admin_ids.insert(id);
        }
        Ok(Collector {
            bot,
            channels,
            admins: admin_ids,
        })
    }

    fn is_watched(&self, chat: &Chat) -> bool {
        self.channels.is_empty()
            || self.channels.contains(&chat.id.to_string())
            || (!chat.username.is_empty()
                && self
                    .channels
                    .contains(&format!("@{}", chat.username.to_lowercase())))
    }

    fn handle(&self, update: laser_tele::Update) {
        match &update.kind {
            UpdateKind::ChannelPost(post) | UpdateKind::EditedChannelPost(post) => {
                self.collect_from_channel(post)
            }
            UpdateKind::Message(message) => self.handle_message(message),
            _ => {}
        }
    }

    fn collect_from_channel(&self, post: &Message) {
        if !self.is_watched(&post.chat) {
            return;
        }
        let texts = with_links(proxy_texts(post));
        if texts.is_empty() {
            return;
        }
        let source = format!("channel:{}/{}", post.chat.id, post.message_id);
        match self.add(&texts, &source) {
            Ok(result) => println!(
                "Post {} of {}: added {}, known {}, errors {}",
                post.message_id,
                post.chat.title,
                result.added,
                result.known,
                result.errors.len()
            ),
            Err(e) => eprintln!("Can't add MTProxy servers: {e}"),
        }
    }

    fn handle_message(&self, message: &Message) {
        if message.chat.kind != "private" {
            return;
        }
        let user_id = message.from.as_ref().map_or(0, |user| user.id);
        if !self.admins.contains(&user_id) {
            return self.reply(
                message,
                &format!(
                    "This bot collects MTProxy servers for its administrators. Your identifier is {user_id}."
                ),
            );
        }

        let command = message.command();
        if !command.is_empty() {
            let args: Vec<&str> = message.text.split_whitespace().skip(1).collect();
            return self.reply(message, &self.run_command(command, &args));
        }

        let texts = proxy_texts(message);
        if texts.is_empty() {
            return self.reply(message, "Send or forward a post with MTProxy links");
        }
        let source = match &message.forward_origin {
            Some(origin) if origin.kind == "channel" => format!(
                "channel:{}/{}",
                origin.chat.as_ref().map_or(0, |chat| chat.id),
                origin.message_id
            ),
            _ => format!("user:{user_id}"),
        };
        match self.add(&texts, &source) {
            Ok(result) => self.reply(message, &describe_add_result(&result)),
            Err(e) => self.reply(message, &format!("Error: {e}")),
        }
    }

    fn run_command(&self, command: &str, args: &[&str]) -> String {
        match command {
            "/proxies" => match self.call::<Registry>("getMTProxies", json!({})) {
                Ok(registry) => describe_registry(&registry),
                Err(e) => format!("Error: {e}"),
            },
            "/check" => match self.call::<Value>("checkMTProxies", json!({})) {
                Ok(_) => "Checking all servers, see /proxies in a minute".to_string(),
                Err(e) => format!("Error: {e}"),
            },
            "/use" | "/remove" => {
                let Some(id) = args.first().and_then(|id| id.parse::<i64>().ok()) else {
                    return format!("Usage: {command} <id from /proxies>");
                };
                let method = if command == "/use" {
                    "setMTProxy"
                } else {
                    "removeMTProxy"
                };
                match self.call::<Value>(method, json!({ "id": id })) {
                    Ok(_) => "Done".to_string(),
                    Err(e) => format!("Error: {e}"),
                }
            }
            _ => "Send or forward posts with MTProxy links, they are added to the registry of the server.\n\
                  /proxies - the registry\n/check - check all servers now\n/use <id> - switch bots to the server\n\
                  /remove <id> - remove the server"
                .to_string(),
        }
    }

    /// Sends the texts with links to the registry of the server
    fn add(&self, texts: &[String], source: &str) -> Result<AddResult, String> {
        self.call("addMTProxies", json!({ "links": texts, "source": source }))
    }

    fn call<T: for<'de> Deserialize<'de>>(&self, method: &str, params: Value) -> Result<T, String> {
        let result = self.bot.call(method, params).map_err(|e| e.to_string())?;
        serde_json::from_value(result).map_err(|e| format!("wrong result of {method}: {e}"))
    }

    fn reply(&self, message: &Message, text: &str) {
        if let Err(e) = self.bot.send_message(message.chat.id, text) {
            eprintln!("Can't send message: {e}");
        }
    }
}

/// The texts of the message, in which the server looks for MTProxy links:
/// the text or the caption, hidden links and URL buttons
fn proxy_texts(message: &Message) -> Vec<String> {
    let mut texts: Vec<String> = [&message.text, &message.caption]
        .into_iter()
        .filter(|text| !text.is_empty())
        .cloned()
        .collect();
    for entity in message.entities.iter().chain(&message.caption_entities) {
        if entity.kind == "text_link" {
            texts.push(entity.url.clone());
        }
    }
    if let Some(keyboard) = &message.reply_markup {
        for button in keyboard.rows.iter().flatten() {
            if !button.url.is_empty() {
                texts.push(button.url.clone());
            }
        }
    }
    texts
}

/// Leaves only texts with MTProxy links, so posts without them aren't sent to the server
fn with_links(texts: Vec<String>) -> Vec<String> {
    texts
        .into_iter()
        .filter(|text| text.to_lowercase().contains("proxy?"))
        .collect()
}

fn describe_add_result(result: &AddResult) -> String {
    let mut text = format!("Added {}, already known {}", result.added, result.known);
    for error in &result.errors {
        text += &format!("\n{}: {}", error.link, error.error);
    }
    text
}

fn describe_registry(registry: &Registry) -> String {
    if registry.proxies.is_empty() {
        return "The registry is empty, send or forward posts with MTProxy links".to_string();
    }
    // the active server first, then working ones by ping
    let rank = |state: &str| match state {
        "working" => 0,
        "unchecked" => 1,
        _ => 2,
    };
    let mut proxies: Vec<&RegistryProxy> = registry.proxies.iter().collect();
    proxies.sort_by(|a, b| {
        b.is_active
            .cmp(&a.is_active)
            .then(rank(&a.state).cmp(&rank(&b.state)))
            .then(a.ping.total_cmp(&b.ping))
    });

    let mut text = format!(
        "Bots connected: {} of {}. Servers: {}",
        registry.connected_bot_count,
        registry.bot_count,
        proxies.len()
    );
    if registry.is_checking {
        text += ", checking now";
    }
    for (i, proxy) in proxies.iter().enumerate() {
        let icon = match proxy.state.as_str() {
            "working" => "✅",
            "unchecked" => "❔",
            _ => "❌",
        };
        let mut line = format!(
            "\n{icon} #{} {}:{} {}",
            proxy.id, proxy.server, proxy.port, proxy.secret_type
        );
        if !proxy.domain.is_empty() {
            line += &format!(" {}", proxy.domain);
        }
        if proxy.state == "working" {
            line += &format!(", {:.0} ms", proxy.ping * 1000.0);
        }
        if proxy.state == "failing" && !proxy.last_error.is_empty() {
            line += &format!(", {}", proxy.last_error);
        }
        if proxy.is_active {
            line += " ← active";
        }
        if text.len() + line.len() > MAX_LIST_LENGTH {
            text += &format!("\n... and {} more", proxies.len() - i);
            break;
        }
        text += &line;
    }
    text
}

fn main() -> laser_tele::Result<()> {
    let api_url =
        std::env::var("BOT_API_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    let bot = blocking::Bot::new(Config {
        api_url,
        timeout: Duration::from_secs(2),
        allowed_updates: vec![
            "message".into(),
            "channel_post".into(),
            "edited_channel_post".into(),
        ],
        ..Default::default()
    })?;
    let env = |name| std::env::var(name).unwrap_or_default();
    let collector = match Collector::new(
        bot.clone(),
        &env("MTPROXY_CHANNELS"),
        &env("MTPROXY_ADMINS"),
    ) {
        Ok(collector) => collector,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    if collector.admins.is_empty() {
        println!("MTPROXY_ADMINS is empty: only posts of channels are collected");
    }
    bot.run(|update| collector.handle(update));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(json: &str) -> Message {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn finds_texts_with_links() {
        let post = message(
            r#"{"message_id":7,"chat":{"id":-100123,"type":"channel"},
            "text":"Fresh: tg://proxy?server=a&port=443&secret=dd00 and Connect",
            "entities":[{"type":"text_link","offset":0,"length":5,"url":"https://t.me/proxy?server=b&port=443&secret=dd11"}],
            "reply_markup":{"inline_keyboard":[[{"text":"Connect","url":"https://t.me/proxy?server=c&port=443&secret=dd22"},
            {"text":"Site","url":"https://example.com"}]]}}"#,
        );
        let texts = with_links(proxy_texts(&post));
        assert_eq!(texts.len(), 3, "{texts:?}");
        assert!(texts[1].contains("server=b") && texts[2].contains("server=c"));

        let caption = message(r#"{"message_id":8,"chat":{"id":1},"caption":"Good morning"}"#);
        assert_eq!(proxy_texts(&caption), vec!["Good morning".to_string()]);
        assert!(with_links(proxy_texts(&caption)).is_empty());
    }

    #[test]
    fn describes_registry() {
        let registry: Registry = serde_json::from_str(
            r#"{"active_id":2,"bot_count":2,"connected_bot_count":1,"proxies":[
            {"id":1,"server":"a.example.com","port":443,"secret_type":"dd","state":"failing","last_error":"Connection closed"},
            {"id":2,"server":"b.example.com","port":443,"secret_type":"ee","domain":"google.com","state":"working","is_active":true,"ping":0.25},
            {"id":3,"server":"c.example.com","port":443,"secret_type":"dd","state":"working","ping":0.1}]}"#,
        )
        .unwrap();
        assert_eq!(
            describe_registry(&registry),
            "Bots connected: 1 of 2. Servers: 3\n✅ #2 b.example.com:443 ee google.com, 250 ms ← active\n\
             ✅ #3 c.example.com:443 dd, 100 ms\n❌ #1 a.example.com:443 dd, Connection closed"
        );

        let long = Registry {
            proxies: (0..200)
                .map(|id| RegistryProxy {
                    id,
                    server: "proxy.example.com".into(),
                    port: 443,
                    state: "failing".into(),
                    last_error: "Connection closed".into(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let text = describe_registry(&long);
        assert!(
            text.len() <= 4096 && text.contains("more"),
            "{}",
            text.len()
        );
    }

    #[test]
    fn describes_add_result() {
        let result: AddResult = serde_json::from_str(
            r#"{"added":1,"known":0,"ids":[3],"errors":[{"link":"tg://proxy?x","error":"wrong port of the MTProxy"}]}"#,
        )
        .unwrap();
        assert_eq!(
            describe_add_result(&result),
            "Added 1, already known 0\ntg://proxy?x: wrong port of the MTProxy"
        );
    }

    #[test]
    fn parses_settings() {
        let bot = blocking::Bot::new(Config {
            api_key: "123456:TEST-KEY".into(),
            log_mode: laser_tele::LogMode::Off,
            ..Default::default()
        })
        .unwrap();
        let collector = Collector::new(bot.clone(), "@FreeProxies, -100777", "137, 500").unwrap();
        let channel = |id, username: &str| Chat {
            id,
            username: username.into(),
            ..Default::default()
        };
        assert!(collector.is_watched(&channel(-100123, "freeproxies")));
        assert!(collector.is_watched(&channel(-100777, "")));
        assert!(!collector.is_watched(&channel(-100555, "other")));
        assert_eq!(collector.admins, HashSet::from([137, 500]));
        assert!(Collector::new(bot, "", "abc").is_err());
    }
}
