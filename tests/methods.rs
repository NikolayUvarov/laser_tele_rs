mod common;

use common::*;
use laser_tele::*;
use serde_json::{Value, json};

/// Checks the method and JSON params of the last request
fn check_request(tg: &FakeTelegram, method: &str, params: Value) {
    let request = tg.last_request();
    assert_eq!(request.method, method);
    assert_eq!(request.json, params, "params of {method}");
}

#[tokio::test]
async fn send_message_sends_text_as_is() {
    let t = new_bot(Config::default());
    t.tg.set_response(
        200,
        r#"{"ok":true,"result":{"message_id":55,"chat":{"id":1,"type":"private"},"text":"x"}}"#,
    );
    let text = "line1\nline2; 100% C# 1+1=2 tom & jerry";
    let sent = t.bot.send_message(-1001234567890, text).await.unwrap();
    assert_eq!(sent.message_id, 55);
    check_request(
        &t.tg,
        "sendMessage",
        json!({ "chat_id": -1001234567890_i64, "text": text }),
    );
}

#[tokio::test]
async fn send_message_with_config_and_edit() {
    let t = new_bot(Config::default());
    let keyboard = InlineKeyboard::new(vec![vec![Button::callback("OK", "ok")]]);
    let config = MessageConfig {
        parse_mode: "HTML".into(),
        reply_to_message_id: 10,
        business_connection_id: "bc1".into(),
        protect_content: true,
        reply_markup: Some(keyboard),
        ..Default::default()
    };
    t.bot
        .send_message_with_config(1, "<b>Hi</b>", &config)
        .await
        .unwrap();
    check_request(
        &t.tg,
        "sendMessage",
        json!({
            "chat_id": 1, "text": "<b>Hi</b>", "parse_mode": "HTML", "reply_parameters": { "message_id": 10 },
            "business_connection_id": "bc1", "protect_content": true,
            "reply_markup": { "inline_keyboard": [[{ "text": "OK", "callback_data": "ok" }]] }
        }),
    );

    t.bot.edit_message_text(1, 55, "edited").await.unwrap();
    check_request(
        &t.tg,
        "editMessageText",
        json!({ "chat_id": 1, "message_id": 55, "text": "edited" }),
    );
    t.bot
        .edit_message_reply_markup(1, 55, &InlineKeyboard::default())
        .await
        .unwrap();
    check_request(
        &t.tg,
        "editMessageReplyMarkup",
        json!({ "chat_id": 1, "message_id": 55, "reply_markup": { "inline_keyboard": [] } }),
    );
}

#[tokio::test]
async fn keyboard_and_buttons() {
    let t = new_bot(Config::default());
    let keyboard = InlineKeyboard::new(vec![
        vec![
            add_button("A & B", "a#b"),
            Button::url("Site", "https://example.com").style("primary"),
        ],
        vec![
            Button::switch_inline_query("Share", "try me"),
            Button::switch_inline_query_current_chat("Search", ""),
        ],
        vec![Button::game("Play"), Button::pay("Pay")],
    ]);
    t.bot.send_keyboard(1, "choose", &keyboard).await.unwrap();
    check_request(
        &t.tg,
        "sendMessage",
        json!({ "chat_id": 1, "text": "choose", "reply_markup": { "inline_keyboard": [
            [{ "text": "A & B", "callback_data": "a#b" }, { "text": "Site", "url": "https://example.com", "style": "primary" }],
            [{ "text": "Share", "switch_inline_query": "try me" }, { "text": "Search" }],
            [{ "text": "Play", "callback_game": {} }, { "text": "Pay", "pay": true }]
        ] } }),
    );
}

#[tokio::test]
async fn callback_query_answers() {
    let t = new_bot(Config::default());
    t.bot.answer_callback_query("q1", "Done!").await.unwrap();
    check_request(
        &t.tg,
        "answerCallbackQuery",
        json!({ "callback_query_id": "q1", "text": "Done!" }),
    );
    t.bot.answer_callback_query("q2", "").await.unwrap();
    check_request(
        &t.tg,
        "answerCallbackQuery",
        json!({ "callback_query_id": "q2" }),
    );
    let config = CallbackAnswerConfig {
        url: "https://example.com/game".into(),
        show_alert: true,
        cache_time: 5,
        ..Default::default()
    };
    t.bot
        .answer_callback_query_with_config("q3", &config)
        .await
        .unwrap();
    check_request(
        &t.tg,
        "answerCallbackQuery",
        json!({ "callback_query_id": "q3", "url": "https://example.com/game", "show_alert": true, "cache_time": 5 }),
    );
}

#[tokio::test]
async fn polls() {
    let t = new_bot(Config::default());
    t.tg.set_response(200, r#"{"ok":true,"result":{"message_id":7,"chat":{"id":1},"poll":{"id":"p1","question":"2+2?","type":"quiz"}}}"#);
    let config = PollConfig {
        quiz: true,
        correct_option_ids: vec![1],
        explanation: "Math".into(),
        not_anonymous: true,
        open_period: 60,
        ..Default::default()
    };
    let sent = t
        .bot
        .send_poll(1, "2+2?", &["3", "4"], &config)
        .await
        .unwrap();
    assert_eq!(sent.poll.unwrap().id, "p1");
    check_request(
        &t.tg,
        "sendPoll",
        json!({ "chat_id": 1, "question": "2+2?", "options": [{ "text": "3" }, { "text": "4" }], "type": "quiz",
                "correct_option_ids": [1], "explanation": "Math", "is_anonymous": false, "open_period": 60 }),
    );

    t.bot
        .send_poll(1, "Tea?", &["Yes", "No"], &PollConfig::default())
        .await
        .unwrap();
    check_request(
        &t.tg,
        "sendPoll",
        json!({ "chat_id": 1, "question": "Tea?", "options": [{ "text": "Yes" }, { "text": "No" }] }),
    );

    t.tg.set_response(200, r#"{"ok":true,"result":{"id":"p1","question":"Tea?","options":[{"text":"Yes","voter_count":3},{"text":"No","voter_count":1}],"total_voter_count":4,"is_closed":true}}"#);
    let poll = t.bot.stop_poll(1, 7).await.unwrap();
    assert!(poll.is_closed);
    assert_eq!(poll.total_voter_count, 4);
    assert_eq!(poll.options[0].voter_count, 3);
    check_request(&t.tg, "stopPoll", json!({ "chat_id": 1, "message_id": 7 }));
}

#[tokio::test]
async fn inline_queries() {
    let t = new_bot(Config::default());
    let results = vec![
        InlineQueryResult::article("1", "Cat", "Meow").set("description", "A cat"),
        InlineQueryResult::photo(
            "2",
            "https://example.com/cat.jpg",
            "https://example.com/thumb.jpg",
        ),
        InlineQueryResult::cached("mpeg4_gif", "3", "file3", ""),
        InlineQueryResult::cached("document", "4", "file4", "Doc"),
        InlineQueryResult::from_json(
            json!({ "type": "venue", "id": "5", "title": "Office", "address": "Main street", "latitude": 1.5, "longitude": 2.5 }),
        ),
    ];
    let config = InlineQueryConfig {
        cache_time: 10,
        is_personal: true,
        next_offset: "20".into(),
        button_text: "Settings".into(),
        button_start_parameter: "settings".into(),
    };
    t.bot
        .answer_inline_query("iq1", &results, &config)
        .await
        .unwrap();
    check_request(
        &t.tg,
        "answerInlineQuery",
        json!({ "inline_query_id": "iq1", "cache_time": 10, "is_personal": true, "next_offset": "20",
                "button": { "text": "Settings", "start_parameter": "settings" }, "results": [
            { "type": "article", "id": "1", "title": "Cat", "description": "A cat", "input_message_content": { "message_text": "Meow" } },
            { "type": "photo", "id": "2", "photo_url": "https://example.com/cat.jpg", "thumbnail_url": "https://example.com/thumb.jpg" },
            { "type": "mpeg4_gif", "id": "3", "mpeg4_file_id": "file3" },
            { "type": "document", "id": "4", "document_file_id": "file4", "title": "Doc" },
            { "type": "venue", "id": "5", "title": "Office", "address": "Main street", "latitude": 1.5, "longitude": 2.5 }
        ] }),
    );

    // no results must be sent as an empty list
    t.bot
        .answer_inline_query("iq2", &[], &InlineQueryConfig::default())
        .await
        .unwrap();
    check_request(
        &t.tg,
        "answerInlineQuery",
        json!({ "inline_query_id": "iq2", "results": [] }),
    );

    t.bot
        .answer_guest_query("gq1", &InlineQueryResult::article("1", "Answer", "42"))
        .await
        .unwrap();
    check_request(
        &t.tg,
        "answerGuestQuery",
        json!({ "guest_query_id": "gq1", "result": { "type": "article", "id": "1", "title": "Answer", "input_message_content": { "message_text": "42" } } }),
    );
}

#[tokio::test]
async fn payments() {
    let t = new_bot(Config::default());
    let invoice = InvoiceConfig {
        title: "Coffee".into(),
        description: "Big cup".into(),
        payload: "order1".into(),
        currency: "XTR".into(),
        prices: vec![LabeledPrice::new("Coffee", 50)],
        start_parameter: "coffee".into(),
        subscription_period: 2592000,
        reply_markup: Some(InlineKeyboard::new(vec![vec![Button::pay("Pay 50 XTR")]])),
        ..Default::default()
    };
    t.bot.send_invoice(1, &invoice).await.unwrap();
    check_request(
        &t.tg,
        "sendInvoice",
        json!({ "chat_id": 1, "title": "Coffee", "description": "Big cup", "payload": "order1", "currency": "XTR",
                "prices": [{ "label": "Coffee", "amount": 50 }], "start_parameter": "coffee",
                "reply_markup": { "inline_keyboard": [[{ "text": "Pay 50 XTR", "pay": true }]] } }),
    );

    t.tg.set_response(200, r#"{"ok":true,"result":"https://t.me/$invoice"}"#);
    assert_eq!(
        t.bot.create_invoice_link(&invoice).await.unwrap(),
        "https://t.me/$invoice"
    );
    check_request(
        &t.tg,
        "createInvoiceLink",
        json!({ "title": "Coffee", "description": "Big cup", "payload": "order1", "currency": "XTR",
                "prices": [{ "label": "Coffee", "amount": 50 }], "subscription_period": 2592000 }),
    );

    t.tg.set_response(200, r#"{"ok":true,"result":true}"#);
    let options = vec![ShippingOption {
        id: "post".into(),
        title: "Post".into(),
        prices: vec![LabeledPrice::new("Delivery", 300)],
    }];
    t.bot
        .answer_shipping_query("sq1", &options, "")
        .await
        .unwrap();
    check_request(
        &t.tg,
        "answerShippingQuery",
        json!({ "shipping_query_id": "sq1", "ok": true, "shipping_options": [{ "id": "post", "title": "Post", "prices": [{ "label": "Delivery", "amount": 300 }] }] }),
    );
    t.bot
        .answer_shipping_query("sq2", &[], "No delivery to Mars")
        .await
        .unwrap();
    check_request(
        &t.tg,
        "answerShippingQuery",
        json!({ "shipping_query_id": "sq2", "ok": false, "error_message": "No delivery to Mars" }),
    );

    t.bot.answer_pre_checkout_query("pq1", "").await.unwrap();
    check_request(
        &t.tg,
        "answerPreCheckoutQuery",
        json!({ "pre_checkout_query_id": "pq1", "ok": true }),
    );
    t.bot
        .answer_pre_checkout_query("pq2", "Out of stock")
        .await
        .unwrap();
    check_request(
        &t.tg,
        "answerPreCheckoutQuery",
        json!({ "pre_checkout_query_id": "pq2", "ok": false, "error_message": "Out of stock" }),
    );

    t.bot.refund_star_payment(5, "ch1").await.unwrap();
    check_request(
        &t.tg,
        "refundStarPayment",
        json!({ "user_id": 5, "telegram_payment_charge_id": "ch1" }),
    );
    t.bot
        .edit_user_star_subscription(5, "ch1", true)
        .await
        .unwrap();
    check_request(
        &t.tg,
        "editUserStarSubscription",
        json!({ "user_id": 5, "telegram_payment_charge_id": "ch1", "is_canceled": true }),
    );
}

#[tokio::test]
async fn games() {
    let t = new_bot(Config::default());
    t.bot.send_game(1, "race").await.unwrap();
    check_request(
        &t.tg,
        "sendGame",
        json!({ "chat_id": 1, "game_short_name": "race" }),
    );

    t.tg.set_response(200, r#"{"ok":true,"result":true}"#);
    t.bot
        .set_game_score(
            5,
            100,
            &GameMessage::Chat {
                chat_id: 1,
                message_id: 7,
            },
            true,
        )
        .await
        .unwrap();
    check_request(
        &t.tg,
        "setGameScore",
        json!({ "user_id": 5, "score": 100, "force": true, "chat_id": 1, "message_id": 7 }),
    );
    t.bot
        .set_game_score(5, 100, &GameMessage::Inline("im1".into()), false)
        .await
        .unwrap();
    check_request(
        &t.tg,
        "setGameScore",
        json!({ "user_id": 5, "score": 100, "inline_message_id": "im1" }),
    );

    t.tg.set_response(200, r#"{"ok":true,"result":[{"position":1,"user":{"id":5,"first_name":"Ann"},"score":100},{"position":2,"user":{"id":6},"score":90}]}"#);
    let scores = t
        .bot
        .get_game_high_scores(
            5,
            &GameMessage::Chat {
                chat_id: 1,
                message_id: 7,
            },
        )
        .await
        .unwrap();
    assert_eq!(scores.len(), 2);
    assert_eq!(scores[0].user.first_name, "Ann");
    assert_eq!(scores[1].score, 90);
    check_request(
        &t.tg,
        "getGameHighScores",
        json!({ "user_id": 5, "chat_id": 1, "message_id": 7 }),
    );
}

#[tokio::test]
async fn reactions_and_chats() {
    let t = new_bot(Config::default());
    t.tg.set_response(200, r#"{"ok":true,"result":true}"#);
    t.bot.set_message_reaction(1, 7, "👍").await.unwrap();
    check_request(
        &t.tg,
        "setMessageReaction",
        json!({ "chat_id": 1, "message_id": 7, "reaction": [{ "type": "emoji", "emoji": "👍" }] }),
    );
    t.bot.set_message_reaction(1, 7, "").await.unwrap();
    check_request(
        &t.tg,
        "setMessageReaction",
        json!({ "chat_id": 1, "message_id": 7, "reaction": [] }),
    );

    t.bot.approve_chat_join_request(-100, 5).await.unwrap();
    check_request(
        &t.tg,
        "approveChatJoinRequest",
        json!({ "chat_id": -100, "user_id": 5 }),
    );
    t.bot.decline_chat_join_request(-100, 6).await.unwrap();
    check_request(
        &t.tg,
        "declineChatJoinRequest",
        json!({ "chat_id": -100, "user_id": 6 }),
    );

    t.tg.set_response(200, r#"{"ok":true,"result":"777:MANAGED"}"#);
    assert_eq!(
        t.bot.get_managed_bot_token(777).await.unwrap(),
        "777:MANAGED"
    );
    check_request(&t.tg, "getManagedBotToken", json!({ "user_id": 777 }));
}

#[tokio::test]
async fn call_any_method() {
    let t = new_bot(Config::default());
    t.tg.set_response(
        200,
        r#"{"ok":true,"result":{"id":1,"is_bot":true,"first_name":"LaserBot"}}"#,
    );
    let me: User = serde_json::from_value(t.bot.call("getMe", Value::Null).await.unwrap()).unwrap();
    assert_eq!(me.first_name, "LaserBot");
    check_request(&t.tg, "getMe", json!({}));

    t.bot
        .call("sendDice", json!({ "chat_id": 1, "emoji": "🎲" }))
        .await
        .unwrap();
    check_request(&t.tg, "sendDice", json!({ "chat_id": 1, "emoji": "🎲" }));
}

#[tokio::test]
async fn api_errors() {
    let t = new_bot(Config::default());
    t.tg.set_response(
        400,
        r#"{"ok":false,"error_code":400,"description":"Bad Request: chat not found"}"#,
    );
    let err = t.bot.send_message(1, "text").await.unwrap_err();
    let api = err.api().expect("ApiError");
    assert_eq!(api.error_code, 400);
    assert_eq!(api.description, "Bad Request: chat not found");
    assert_eq!(api.method, "sendMessage");
    assert_eq!(
        err.to_string(),
        "telegram sendMessage: 400 Bad Request: chat not found"
    );

    t.tg.set_response(429, r#"{"ok":false,"error_code":429,"description":"Too Many Requests: retry after 5","parameters":{"retry_after":5}}"#);
    let err = t.bot.send_message(1, "text").await.unwrap_err();
    assert_eq!(err.api().unwrap().retry_after, 5);

    t.tg.set_response(502, "<html>Bad Gateway</html>");
    let err = t.bot.send_message(1, "text").await.unwrap_err();
    assert_eq!(err.api().unwrap().error_code, 502);
}

#[tokio::test]
async fn network_errors_hide_the_token() {
    let dir = tempfile::tempdir().unwrap();
    let bot = Bot::new(Config {
        api_key: TEST_KEY.into(),
        api_url: "http://127.0.0.1:1".into(),
        log_dir: dir.path().into(),
        ..Default::default()
    })
    .unwrap();
    let err = bot.send_message(1, "text").await.unwrap_err();
    assert!(matches!(err, Error::Network(_)), "{err:?}");
    assert!(!err.to_string().contains(TEST_KEY), "{err}");
    assert!(!format!("{err:?}").contains(TEST_KEY), "{err:?}");
    let log = std::fs::read_to_string(dir.path().join("sendMessage.log")).unwrap();
    assert!(
        log.contains("CONNECTION_ERROR") && !log.contains(TEST_KEY),
        "{log}"
    );
}

#[tokio::test]
async fn send_files() {
    let t = new_bot(Config::default());
    let file = t.dir.path().join("test.jpg");
    std::fs::write(&file, "image data").unwrap();
    t.bot
        .send_photo(-100123, "Test image & more", &file)
        .await
        .unwrap();
    let request = t.tg.last_request();
    assert_eq!(request.method, "sendPhoto");
    assert_eq!(request.form["chat_id"], "-100123");
    assert_eq!(request.form["caption"], "Test image & more");
    assert_eq!(
        (request.file_field.as_str(), request.file_name.as_str()),
        ("photo", "test.jpg")
    );
    assert_eq!(request.file_content, b"image data");

    t.bot.send_document(1, "", &file).await.unwrap();
    let request = t.tg.last_request();
    assert_eq!(
        (request.method.as_str(), request.file_field.as_str()),
        ("sendDocument", "document")
    );
    assert!(!request.form.contains_key("caption"));

    let err = t
        .bot
        .send_video(1, "", "no_such_file.mp4")
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Io(e) if e.kind() == std::io::ErrorKind::NotFound),
        "{err:?}"
    );
}

#[tokio::test]
async fn load_files() {
    let t = new_bot(Config::default());
    t.tg.add_file("files/good", "image data");
    let path = t.bot.load_file("good").await.unwrap();
    assert_eq!(path, t.download_dir().join("files/good"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "image data");

    let err = t.bot.load_file("wrong").await.unwrap_err();
    assert_eq!(err.api().unwrap().error_code, 400);
    let err = t.bot.load_file("deleted").await.unwrap_err();
    assert_eq!(err.api().unwrap().error_code, 404);
    assert!(!err.to_string().contains(TEST_KEY));

    let link = format!("{}/file/bot{TEST_KEY}/files/good", t.tg.url);
    let path = t.bot.file_download(&link, "copy/good.jpg").await.unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), "image data");
}

#[tokio::test]
async fn logs_without_content_by_default() {
    let t = new_bot(Config::default());
    t.bot.update_request(|_| async {}).await;
    t.tg.add_updates(vec![text_update(794872550, "secret update")]);
    t.bot.update_request(|_| async {}).await;
    t.bot.send_message(137, "secret text").await.unwrap();

    let send_log = t.read_log("sendMessage");
    assert!(
        send_log.contains("sendMessage   REQV: POST sendMessage chat_id=137"),
        "{send_log}"
    );
    assert!(
        send_log.contains("RESP: 200 OK") && !send_log.contains("secret"),
        "{send_log}"
    );
    let updates_log = t.read_log("updateRequest");
    assert!(
        updates_log.contains("UPDATES: 794872550 message") && !updates_log.contains("secret"),
        "{updates_log}"
    );
    assert!(!send_log.contains(TEST_KEY) && !updates_log.contains(TEST_KEY));
}

#[tokio::test]
async fn full_logs_and_no_logs() {
    let t = new_bot(Config {
        log_mode: LogMode::Full,
        ..Default::default()
    });
    t.bot.send_message(137, "secret text").await.unwrap();
    let log = t.read_log("sendMessage");
    assert!(
        log.contains("secret text") && log.contains("<APIKEY>") && !log.contains(TEST_KEY),
        "{log}"
    );

    let t = new_bot(Config {
        log_mode: LogMode::Off,
        ..Default::default()
    });
    t.bot.send_message(137, "secret text").await.unwrap();
    assert!(!t.log_dir().exists());
}

#[tokio::test]
async fn logs_are_rotated() {
    let t = new_bot(Config {
        log_max_size: 300,
        ..Default::default()
    });
    for _ in 0..20 {
        t.bot.send_message(137, "text").await.unwrap();
    }
    let size = std::fs::metadata(t.log_dir().join("sendMessage.log"))
        .unwrap()
        .len();
    assert!(size < 300 + 200, "the log is not rotated: {size} bytes");
    assert!(t.log_dir().join("sendMessage.log.1").exists());
    assert_eq!(std::fs::read_dir(t.log_dir()).unwrap().count(), 2);
}

#[tokio::test]
async fn two_bots_in_one_program() {
    let one = new_bot(Config {
        api_key: "111:KEY-ONE".into(),
        ..Default::default()
    });
    let two = new_bot(Config {
        api_key: "222:KEY-TWO".into(),
        ..Default::default()
    });
    one.bot.update_request(|_| async {}).await;
    two.bot.update_request(|_| async {}).await;

    one.tg.add_updates(vec![text_update(100, "to one")]);
    two.tg.add_updates(vec![
        text_update(500, "to two"),
        text_update(501, "to two again"),
    ]);
    let mut got_one = Vec::new();
    one.bot
        .update_request(|u| {
            got_one.push(u.message().unwrap().text.clone());
            async {}
        })
        .await;
    let mut got_two = Vec::new();
    two.bot
        .update_request(|u| {
            got_two.push(u.message().unwrap().text.clone());
            async {}
        })
        .await;
    assert_eq!(got_one, ["to one"]);
    assert_eq!(got_two, ["to two", "to two again"]);

    one.bot.send_message(1, "from one").await.unwrap();
    two.bot.send_message(2, "from two").await.unwrap();
    assert_eq!(one.tg.last_request().json["text"], "from one");
    assert_eq!(two.tg.last_request().json["text"], "from two");

    // the same file path of different bots is saved to different directories
    one.tg.add_file("files/doc1", "one");
    two.tg.add_file("files/doc1", "two");
    let path_one = one.bot.load_file("doc1").await.unwrap();
    let path_two = two.bot.load_file("doc1").await.unwrap();
    assert_ne!(path_one, path_two);
    assert_eq!(std::fs::read_to_string(path_two).unwrap(), "two");

    let log_one = one.read_log("sendMessage");
    assert!(
        log_one.contains("chat_id=1") && !log_one.contains("111:KEY-ONE"),
        "{log_one}"
    );
}

#[test]
fn blocking_methods_work() {
    let dir = tempfile::tempdir().unwrap();
    let tg = FakeTelegram::start(TEST_KEY);
    let bot = blocking::Bot::new(test_config(&tg, &dir, TEST_KEY, Config::default())).unwrap();
    bot.send_message(1, "blocking").unwrap();
    assert_eq!(tg.last_request().json["text"], "blocking");
    bot.send_poll(1, "Q", &["A", "B"], &PollConfig::default())
        .unwrap();
    assert_eq!(tg.last_request().method, "sendPoll");
    tg.set_response(200, r#"{"ok":true,"result":true}"#);
    bot.answer_pre_checkout_query("pq1", "").unwrap();
    assert_eq!(tg.last_request().method, "answerPreCheckoutQuery");
    tg.set_response(
        400,
        r#"{"ok":false,"error_code":400,"description":"Bad Request: chat not found"}"#,
    );
    assert_eq!(
        bot.send_message(1, "x")
            .unwrap_err()
            .api()
            .unwrap()
            .error_code,
        400
    );
}

#[test]
fn futures_are_send() {
    fn assert_send<T: Send>(_: T) {}
    let bot = Bot::new(Config {
        api_key: TEST_KEY.into(),
        log_mode: LogMode::Off,
        ..Default::default()
    })
    .unwrap();
    assert_send(bot.send_message(1, "x"));
    assert_send(bot.send_photo(1, "x", "file.jpg"));
    assert_send(bot.send_poll(1, "q", &["a"], &PollConfig::default()));
    assert_send(bot.update_request(|_| async {}));
    assert_send(laser_tele::send_message(1, "x"));
}
