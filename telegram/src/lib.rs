// use ic_cdk::export::Principal;
use ic_cdk::api::*;
use ic_cdk_macros::*;

use ic_cdk::export::candid::CandidType;
use serde::{Deserialize};

// The HTTP Gateway interfaces
// Taken from https://docs.rs/ic-utils/0.14.0/src/ic_utils/interfaces/http_request.rs.html
// and simplified (no streaming_strategy needed).

/// A key-value pair for a HTTP header.
#[derive(CandidType, Deserialize)]
pub struct HeaderField(pub String, pub String);

/// The important components of an HTTP request.
#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
    /// The HTTP method string.
    pub method: String,
    /// The URL that was visited.
    pub url: String,
    /// The request headers.
    pub headers: Vec<HeaderField>,
    /// The request body.
    // #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

/// A HTTP response.
#[derive(CandidType)]
pub struct HttpResponse {
    /// The HTTP status code.
    pub status_code: u16,
    /// The response header map.
    pub headers: Vec<HeaderField>,
    /// The response body.
    // #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    /// Whether the query call should be upgraded to an update call.
    pub upgrade: Option<bool>,
}

// Some application logic

fn get_info() -> String {
    format!("This is a Telegram bot on the Internet Computer!\nMy canister id: {}\nLocal time is {}ns.\nMy cycle balance is {}\nVisit my homepages:\nhttps://t.me/InternetComputerBot\nhttps://{}.raw.ic0.app/\nhttps://github.com/nomeata/ic-telegram-bot", id(), time(), canister_balance(), id())
}

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref JOKES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(vec![
        "What does Mr. Williams reign over? His dom-minions!".to_string()
    ]);
}
fn add_joke(new_joke: String) {
    JOKES.lock().unwrap().push(new_joke);
}
fn get_random_joke() -> (String, usize, usize) {
    let jokes = JOKES.lock().unwrap();
    let n = jokes.len();
    let idx = time() as usize % jokes.len();
    let joke = jokes[idx].clone();
    (joke, idx + 1, n)
}

// Main entry points and dispatchers

#[update]
fn http_request_update(req: HttpRequest) -> HttpResponse {
    dispatch(req)
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    dispatch(req)
}

fn dispatch(req: HttpRequest) -> HttpResponse {
    let uri = req.url.clone();
    match uri.strip_prefix("/webhook/") {
        Some(token) => handle_telegram(token, req),
        None => {
            if req.url == "/" {
                index(req)
            } else {
                err404(req)
            }
        }
    }
}

// Lets take donations
// NB: We are not returning the expected candid here; lets see what happens.
#[update]
fn wallet_receive() -> () {
    let amount = ic_cdk::api::call::msg_cycles_available();
    if amount > 0 {
        ic_cdk::api::call::msg_cycles_accept(amount);
    }
}

// A common handlers

fn ok200() -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(String::from("content-type"), String::from("text/html"))],
        body: "Nothing to do".as_bytes().to_vec(),
        upgrade: Some(false),
    }
}

fn index(_req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(String::from("content-type"), String::from("text/plain"))],
        body: get_info().as_bytes().to_vec(),
        upgrade: Some(false),
    }
}
fn err404(req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 404,
        headers: vec![],
        body: format!(
            "Nothing found at {}\n(but still, you reached the internet computer!)",
            req.url
        )
        .as_bytes()
        .to_vec(),
        upgrade: Some(false),
    }
}

// Telegram boiler plate

use serde_json::Value;
use telegram_bot_raw::{MessageChat, MessageKind, SendMessage, Update, UpdateKind};

fn handle_telegram(_token: &str, req: HttpRequest) -> HttpResponse {
    match serde_json::from_slice::<Update>(&req.body) {
        Err(err) => HttpResponse {
            status_code: 500,
            headers: vec![],
            body: format!("{}", err).as_bytes().to_vec(),
            upgrade: Some(false),
        },
        Ok(update) => match update.kind {
            UpdateKind::Message(msg) => match msg.kind {
                MessageKind::Text { data, .. } => handle_message(msg.chat, data),
                _ => ok200(),
            },
            _ => ok200(),
        },
    }
}

fn add_method(value: &mut Value, method: String) {
    match value {
        Value::Object(m) => {
            m.insert("method".to_string(), Value::String(method));
        }
        _ => (),
    }
}

fn send_message(chat: MessageChat, text: String) -> HttpResponse {
    let m = SendMessage::new(chat, text);
    let mut value = serde_json::to_value(m).unwrap();
    add_method(&mut value, "sendMessage".to_string());
    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(String::from("content-type"), String::from("application/json"))],
        body: serde_json::to_vec(&value).unwrap(),
        upgrade: Some(false),
    }
}

// The actual handler

fn handle_message(chat: MessageChat, text: String) -> HttpResponse {
    match text.as_str() {
        "/start" => send_message(
            chat,
            "Hello! I am a Telegram Bot in a canister. Try /joke, /info".to_string(),
        ),
        "/joke" => {
            let (joke, idx, n) = get_random_joke();
            send_message(
            chat,
            format!(
                "{}\n(This was joke {} of {}. Got a better one? Tell me about it, with /telljoke …!)",
                joke, idx, n
            ),
        )
        }
        "/telljoke" => send_message(chat, "Put the joke after /telljoke!".to_string()),
        "/info" => send_message(chat, get_info()),
        _ => match text.strip_prefix("/telljoke ") {
            Some(joke) => {
                add_joke(joke.to_string());
                let mut resp = send_message(chat, "Ha! Ha! Duly noted.".to_string());
                resp.upgrade = Some(true);
                resp
            }
            _ => send_message(chat, format!("What do you mean, {}?", text)),
        },
    }
}
