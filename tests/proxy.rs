//! Proxies are set in the environment, so their tests are in one function of a separate test binary

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use laser_tele::{Config, Error, LogMode, blocking};

const TEST_KEY: &str = "123456:TEST-KEY";

/// A fake proxy: sends the request line and the Proxy-Authorization header of every connection, refuses them
fn start_proxy() -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                    break;
                }
                let line = line.trim_end();
                if request.is_empty() {
                    request = line.to_string();
                } else if line.to_lowercase().starts_with("proxy-authorization") {
                    request = format!("{request} | {line}");
                }
            }
            let _ = stream.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n");
            let _ = tx.send(request);
        }
    });
    (address.to_string(), rx)
}

fn bot(proxy: &str) -> laser_tele::Result<blocking::Bot> {
    blocking::Bot::new(Config {
        api_key: TEST_KEY.into(),
        log_mode: LogMode::Off,
        proxy: proxy.into(),
        ..Default::default()
    })
}

#[test]
fn proxies() {
    // Config::proxy with a password
    let (address, requests) = start_proxy();
    let err = bot(&format!("http://user:secret@{address}"))
        .unwrap()
        .send_message(1, "text")
        .unwrap_err();
    assert!(matches!(err, Error::Network(_)), "{err:?}");
    let request = requests.recv().unwrap();
    assert!(
        request.starts_with("CONNECT api.telegram.org:443"),
        "{request}"
    );
    assert!(request.ends_with("Basic dXNlcjpzZWNyZXQ="), "{request}");

    // the HTTPS_PROXY environment variable
    let (address, requests) = start_proxy();
    // SAFETY: the only test of this binary, no other threads read the environment
    unsafe {
        std::env::set_var("HTTPS_PROXY", format!("http://{address}"));
        std::env::remove_var("NO_PROXY");
        std::env::remove_var("no_proxy");
    }
    assert!(bot("").unwrap().send_message(1, "text").is_err());
    assert!(
        requests
            .recv()
            .unwrap()
            .starts_with("CONNECT api.telegram.org:443")
    );

    // a wrong proxy doesn't show the password
    let err = bot("http://user:secret@[wrong")
        .err()
        .expect("a wrong proxy must be an error");
    assert!(matches!(err, Error::Config(_)), "{err:?}");
    assert!(!err.to_string().contains("secret"), "{err}");

    // the token and the password are hidden in the debug output of the config
    let config = Config {
        api_key: TEST_KEY.into(),
        proxy: "http://user:secret@proxy:8080".into(),
        ..Default::default()
    };
    let debug = format!("{config:?}");
    assert!(
        !debug.contains(TEST_KEY) && !debug.contains("secret"),
        "{debug}"
    );
}
