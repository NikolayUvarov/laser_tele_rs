mod common;

use std::sync::{Arc, Mutex};

use common::*;
use laser_tele::{ALL_UPDATE_TYPES, Config, Update, UpdateKind, blocking};
use serde_json::{Value, json};

async fn collect(bot: &laser_tele::Bot) -> Vec<Update> {
    let mut got = Vec::new();
    bot.update_request(|update| {
        got.push(update);
        async {}
    })
    .await;
    got
}

#[tokio::test]
async fn first_request_skips_old_updates() {
    let t = new_bot(Config::default());
    t.tg.add_updates(vec![
        text_update(794872550, "ddd"),
        text_update(794872551, "asd"),
        text_update(794872552, "asdfa"),
    ]);
    assert!(collect(&t.bot).await.is_empty());
    assert_eq!(t.tg.last_request().json["offset"], json!(-1));

    // the last skipped update is confirmed by the next request
    assert!(collect(&t.bot).await.is_empty());
    assert_eq!(t.tg.last_request().json["offset"], json!(794872553));
}

#[tokio::test]
async fn new_updates_are_confirmed() {
    let t = new_bot(Config::default());
    t.tg.add_updates(vec![text_update(794872550, "old")]);
    collect(&t.bot).await;

    // no new updates: must not panic and must not repeat old updates
    assert!(collect(&t.bot).await.is_empty());

    t.tg.add_updates(vec![
        text_update(794872551, "first"),
        text_update(794872552, "second"),
    ]);
    let got = collect(&t.bot).await;
    let texts: Vec<&str> = got
        .iter()
        .map(|u| u.message().unwrap().text.as_str())
        .collect();
    assert_eq!(texts, ["first", "second"]);
    assert_eq!(got[1].update_id, 794872552);
    assert_eq!(got[1].message().unwrap().chat.id, 137511897);

    assert!(collect(&t.bot).await.is_empty());
    assert_eq!(t.tg.last_request().json["offset"], json!(794872553));
}

#[tokio::test]
async fn first_message_after_empty_start_is_not_lost() {
    let t = new_bot(Config::default());
    collect(&t.bot).await;
    t.tg.add_updates(vec![text_update(794872550, "hello")]);
    let got = collect(&t.bot).await;
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].message().unwrap().text, "hello");
}

#[tokio::test]
async fn all_update_kinds_are_parsed() {
    let user = test_user();
    let chat = test_chat();
    let message = test_message(1, "text");
    let member = json!({ "status": "member", "user": user });
    let boost_source = json!({ "source": "premium", "user": user });
    let objects: Vec<(&str, Value)> = vec![
        ("message", message.clone()),
        ("edited_message", message.clone()),
        ("channel_post", message.clone()),
        ("edited_channel_post", message.clone()),
        (
            "business_connection",
            json!({ "id": "bc1", "user": user, "user_chat_id": 5, "date": 1, "is_enabled": true, "rights": { "can_reply": true } }),
        ),
        ("business_message", message.clone()),
        ("edited_business_message", message.clone()),
        (
            "deleted_business_messages",
            json!({ "business_connection_id": "bc1", "chat": chat, "message_ids": [3, 4] }),
        ),
        ("guest_message", message.clone()),
        (
            "message_reaction",
            json!({ "chat": chat, "message_id": 7, "user": user, "date": 1, "old_reaction": [], "new_reaction": [{ "type": "emoji", "emoji": "👍" }] }),
        ),
        (
            "message_reaction_count",
            json!({ "chat": chat, "message_id": 7, "date": 1, "reactions": [{ "type": { "type": "emoji", "emoji": "🔥" }, "total_count": 3 }] }),
        ),
        (
            "inline_query",
            json!({ "id": "iq1", "from": user, "query": "cats", "offset": "" }),
        ),
        (
            "chosen_inline_result",
            json!({ "result_id": "r1", "from": user, "query": "cats" }),
        ),
        (
            "callback_query",
            json!({ "id": "cq1", "from": user, "chat_instance": "1", "game_short_name": "race" }),
        ),
        (
            "shipping_query",
            json!({ "id": "sq1", "from": user, "invoice_payload": "order1", "shipping_address": { "country_code": "RU", "city": "Moscow" } }),
        ),
        (
            "pre_checkout_query",
            json!({ "id": "pq1", "from": user, "currency": "XTR", "total_amount": 50, "invoice_payload": "order1" }),
        ),
        (
            "purchased_paid_media",
            json!({ "from": user, "paid_media_payload": "media1" }),
        ),
        (
            "poll",
            json!({ "id": "p1", "question": "Tea?", "options": [{ "persistent_id": "o1", "text": "Yes", "voter_count": 2 }], "total_voter_count": 2, "is_closed": true, "type": "quiz", "correct_option_ids": [0] }),
        ),
        (
            "poll_answer",
            json!({ "poll_id": "p1", "user": user, "option_ids": [1], "option_persistent_ids": ["o2"] }),
        ),
        (
            "my_chat_member",
            json!({ "chat": chat, "from": user, "date": 1, "old_chat_member": member, "new_chat_member": { "status": "kicked", "user": user } }),
        ),
        (
            "chat_member",
            json!({ "chat": chat, "from": user, "date": 1, "old_chat_member": member, "new_chat_member": { "status": "administrator", "user": user, "can_pin_messages": true } }),
        ),
        (
            "chat_join_request",
            json!({ "chat": chat, "from": user, "user_chat_id": 5, "date": 1, "bio": "hello" }),
        ),
        (
            "chat_boost",
            json!({ "chat": chat, "boost": { "boost_id": "b1", "add_date": 1, "expiration_date": 2, "source": boost_source } }),
        ),
        (
            "removed_chat_boost",
            json!({ "chat": chat, "boost_id": "b1", "remove_date": 2, "source": boost_source }),
        ),
        (
            "managed_bot",
            json!({ "user": user, "bot": { "id": 777, "is_bot": true, "first_name": "Managed" } }),
        ),
        (
            "subscription",
            json!({ "user": user, "invoice_payload": "sub1", "state": "canceled" }),
        ),
        (
            "stopped_message_generation",
            json!({ "chat": chat, "draft_id": 42 }),
        ),
    ];
    assert_eq!(
        objects.len(),
        ALL_UPDATE_TYPES.len(),
        "the test must check all kinds of updates"
    );

    let t = new_bot(Config::default());
    collect(&t.bot).await;
    t.tg.add_updates(
        objects
            .iter()
            .enumerate()
            .map(|(num, (kind, object))| test_update(num as i64 + 1, kind, object.clone()))
            .collect(),
    );
    let got = collect(&t.bot).await;
    assert_eq!(got.len(), ALL_UPDATE_TYPES.len());

    for (update, kind) in got.iter().zip(ALL_UPDATE_TYPES) {
        assert_eq!(update.type_name(), *kind);
        assert!(
            !matches!(update.kind, UpdateKind::Unknown(_)),
            "{kind} is not parsed"
        );
    }

    use UpdateKind as K;
    let check = |num: usize, ok: bool| {
        assert!(
            ok,
            "update {} is parsed wrong: {:?}",
            ALL_UPDATE_TYPES[num], got[num]
        )
    };
    for (num, update) in got.iter().enumerate() {
        match &update.kind {
            K::Message(m) | K::EditedMessage(m) | K::ChannelPost(m) | K::EditedChannelPost(m) => {
                check(num, m.text == "text")
            }
            K::BusinessMessage(m) | K::EditedBusinessMessage(m) | K::GuestMessage(m) => {
                check(num, m.text == "text")
            }
            K::BusinessConnection(c) => {
                check(num, c.id == "bc1" && c.rights.as_ref().unwrap().can_reply)
            }
            K::DeletedBusinessMessages(d) => check(num, d.message_ids == [3, 4]),
            K::MessageReaction(r) => check(num, r.new_reaction[0].emoji == "👍"),
            K::MessageReactionCount(r) => check(
                num,
                r.reactions[0].total_count == 3 && r.reactions[0].reaction.emoji == "🔥",
            ),
            K::InlineQuery(q) => check(num, q.query == "cats"),
            K::ChosenInlineResult(r) => check(num, r.result_id == "r1"),
            K::CallbackQuery(q) => check(num, q.game_short_name == "race"),
            K::ShippingQuery(q) => check(num, q.shipping_address.city == "Moscow"),
            K::PreCheckoutQuery(q) => check(num, q.total_amount == 50),
            K::PurchasedPaidMedia(p) => check(num, p.paid_media_payload == "media1"),
            K::Poll(p) => check(
                num,
                p.options[0].voter_count == 2 && p.correct_option_ids == [0] && p.kind == "quiz",
            ),
            K::PollAnswer(a) => check(num, a.option_ids == [1]),
            K::MyChatMember(m) => check(num, m.new_chat_member.status == "kicked"),
            K::ChatMember(m) => check(num, m.new_chat_member.can_pin_messages),
            K::ChatJoinRequest(r) => check(num, r.bio == "hello"),
            K::ChatBoost(b) => check(num, b.boost.source.source == "premium"),
            K::RemovedChatBoost(b) => check(num, b.boost_id == "b1"),
            K::ManagedBot(m) => check(num, m.bot.id == 777),
            K::Subscription(s) => check(num, s.state == "canceled"),
            K::StoppedMessageGeneration(s) => check(num, s.draft_id == 42),
            other => panic!("unexpected kind {other:?}"),
        }
    }
}

#[tokio::test]
async fn unknown_kinds_are_named_and_kept_raw() {
    let t = new_bot(Config::default());
    collect(&t.bot).await;
    t.tg.add_updates(vec![test_update(
        1,
        "future_update",
        json!({ "id": "unknown" }),
    )]);
    let got = collect(&t.bot).await;
    assert_eq!(got[0].type_name(), "future_update");
    assert_eq!(got[0].kind, UpdateKind::Unknown("future_update".into()));
    assert_eq!(got[0].raw["future_update"]["id"], "unknown");
}

#[tokio::test]
async fn message_fields() {
    let t = new_bot(Config::default());
    collect(&t.bot).await;
    let mut photo = test_message(10, "");
    photo["caption"] = json!("my photo");
    photo["photo"] = json!([{ "file_id": "small", "width": 90, "height": 90 }, { "file_id": "big", "width": 1280, "height": 960 }]);
    photo["reply_to_message"] = test_message(9, "original");
    photo["forward_origin"] =
        json!({ "type": "user", "date": 1692347000, "sender_user": test_user() });
    photo["successful_payment"] = json!({ "currency": "XTR", "total_amount": 50, "invoice_payload": "order1", "telegram_payment_charge_id": "ch1" });
    photo["location"] = json!({ "latitude": 55.75, "longitude": 37.62 });
    photo["reply_markup"] = json!({ "inline_keyboard": [[{ "text": "Yes", "callback_data": "yes" }, { "text": "Site", "url": "https://example.com" }]] });
    let mut group = test_message(12, "/start@my_bot promo");
    group["chat"] = json!({ "id": -100555, "type": "supergroup", "title": "Laser team" });
    group["entities"] = json!([{ "type": "bot_command", "offset": 0, "length": 13 }]);
    t.tg.add_updates(vec![
        test_update(1, "message", photo),
        test_update(2, "message", group),
    ]);

    let got = collect(&t.bot).await;
    let photo = got[0].message().unwrap();
    assert_eq!(photo.caption, "my photo");
    assert_eq!(photo.photo[1].file_id, "big");
    assert_eq!(photo.photo[1].width, 1280);
    assert_eq!(photo.reply_to_message.as_ref().unwrap().text, "original");
    assert_eq!(
        photo
            .forward_origin
            .as_ref()
            .unwrap()
            .sender_user
            .as_ref()
            .unwrap()
            .username,
        "nikolosu"
    );
    assert_eq!(
        photo
            .successful_payment
            .as_ref()
            .unwrap()
            .telegram_payment_charge_id,
        "ch1"
    );
    assert_eq!(photo.location.as_ref().unwrap().longitude, 37.62);
    assert_eq!(
        photo.reply_markup.as_ref().unwrap().rows[0][1].url,
        "https://example.com"
    );
    assert!(photo.video.is_none() && photo.text.is_empty());

    let group = got[1].message().unwrap();
    assert_eq!(group.chat.title, "Laser team");
    assert_eq!(group.chat.kind, "supergroup");
    assert_eq!(group.command(), "/start");
    assert_eq!(group.entity_text(&group.entities[0]), "/start@my_bot");
}

#[test]
fn entity_text_counts_utf16() {
    let message: laser_tele::Message = serde_json::from_value(json!({
        "text": "Привет 👋 @nikolosu!",
        "entities": [{ "type": "mention", "offset": 10, "length": 9 }]
    }))
    .unwrap();
    assert_eq!(message.entity_text(&message.entities[0]), "@nikolosu");
    assert_eq!(message.command(), "");
}

#[tokio::test]
async fn allowed_updates_are_requested() {
    let t = new_bot(Config::default());
    collect(&t.bot).await;
    let expected: Vec<Value> = ALL_UPDATE_TYPES.iter().map(|kind| json!(kind)).collect();
    assert_eq!(t.tg.last_request().json["allowed_updates"], json!(expected));

    let t = new_bot(Config {
        allowed_updates: vec!["message".into(), "poll_answer".into()],
        ..Default::default()
    });
    collect(&t.bot).await;
    assert_eq!(
        t.tg.last_request().json["allowed_updates"],
        json!(["message", "poll_answer"])
    );
}

#[tokio::test]
async fn config_callback_channel_and_handler_get_updates_in_order() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let config_order = order.clone();
    let t = new_bot(Config::default().on_update(move |update| {
        config_order
            .lock()
            .unwrap()
            .push(format!("config:{}", update.message().unwrap().text));
    }));
    let mut updates = t.bot.make_chan();
    collect(&t.bot).await;

    t.tg.add_updates(vec![text_update(794872550, "hello")]);
    let bot = t.bot.clone();
    let handler_order = order.clone();
    let poller = tokio::spawn(async move {
        bot.update_request(move |update| {
            handler_order
                .lock()
                .unwrap()
                .push(format!("handler:{}", update.message().unwrap().text));
            async {}
        })
        .await;
    });
    let update = updates.recv().await.unwrap();
    poller.await.unwrap();
    assert_eq!(update.message().unwrap().text, "hello");
    assert_eq!(*order.lock().unwrap(), ["config:hello", "handler:hello"]);
}

#[test]
fn blocking_bot_receives_updates() {
    let dir = tempfile::tempdir().unwrap();
    let tg = FakeTelegram::start(TEST_KEY);
    let bot = blocking::Bot::new(test_config(&tg, &dir, TEST_KEY, Config::default())).unwrap();
    let updates = bot.make_chan();
    bot.update_request(|_| {});

    tg.add_updates(vec![text_update(1, "first"), text_update(2, "second")]);
    let reader = std::thread::spawn(move || {
        let first = updates.recv().unwrap();
        let second = updates.recv().unwrap();
        vec![
            first.message().unwrap().text.clone(),
            second.message().unwrap().text.clone(),
        ]
    });
    let mut handled = Vec::new();
    bot.update_request(|update| handled.push(update.message().unwrap().text.clone()));
    assert_eq!(handled, ["first", "second"]);
    assert_eq!(reader.join().unwrap(), ["first", "second"]);
}

#[test]
fn update_round_trips_as_json() {
    let raw = text_update(5, "hi");
    let update: Update = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(serde_json::to_value(&update).unwrap(), raw);
}
