//! The default bot is global, so its tests are in one function of a separate test binary

mod common;

use std::sync::{Arc, Mutex};

use common::*;
use laser_tele::{Config, Error, blocking};

#[test]
fn default_bot_functions() {
    // SAFETY: the only test of this binary, no other threads read the environment
    unsafe { std::env::remove_var("TG_API_KEY") };
    assert!(
        !std::path::Path::new(".APIKEY").exists(),
        "the test needs no .APIKEY file in the package directory"
    );

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // without a token functions return an error instead of stopping the program
    let err = runtime
        .block_on(laser_tele::send_message(1, "text"))
        .unwrap_err();
    assert!(matches!(err, Error::NoToken), "{err:?}");
    assert!(matches!(
        blocking::send_message(1, "text").unwrap_err(),
        Error::NoToken
    ));

    let dir = tempfile::tempdir().unwrap();
    let tg = FakeTelegram::start(TEST_KEY);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen_by_config = seen.clone();
    let config = Config::default()
        .on_update(move |update| seen_by_config.lock().unwrap().push(update.update_id));
    laser_tele::init(test_config(&tg, &dir, TEST_KEY, config)).unwrap();

    // async functions of the default bot
    runtime.block_on(async {
        laser_tele::send_message(5, "async").await.unwrap();
        assert_eq!(tg.last_request().json["text"], "async");

        let mut updates = laser_tele::make_chan().unwrap();
        laser_tele::update_request(|_| async {}).await.unwrap();
        tg.add_updates(vec![text_update(1, "hello")]);
        let mut handled = Vec::new();
        laser_tele::update_request(|update| {
            handled.push(update.update_id);
            async {}
        })
        .await
        .unwrap();
        assert_eq!(handled, [1]);
        assert_eq!(updates.recv().await.unwrap().update_id, 1);
    });

    // blocking functions share the default bot: the update 1 is already confirmed
    blocking::send_message(6, "blocking").unwrap();
    assert_eq!(tg.last_request().json["text"], "blocking");
    tg.add_updates(vec![text_update(2, "again")]);
    let updates = blocking::make_chan().unwrap();
    let reader = std::thread::spawn(move || updates.recv().unwrap().update_id);
    let mut handled = Vec::new();
    blocking::update_request(|update| handled.push(update.update_id)).unwrap();
    assert_eq!(handled, [2]);
    assert_eq!(reader.join().unwrap(), 2);
    assert_eq!(*seen.lock().unwrap(), [1, 2]);

    // init replaces the default bot
    let tg2 = FakeTelegram::start("222:SECOND");
    laser_tele::init(test_config(&tg2, &dir, "222:SECOND", Config::default())).unwrap();
    blocking::send_message(7, "second").unwrap();
    assert_eq!(tg2.last_request().json["text"], "second");
}
