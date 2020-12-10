// use ic_cdk::export::Principal;
use ic_cdk::api::*;
use ic_cdk_macros::*;

use ic_cdk::export::candid::CandidType;
use serde::{Deserialize, Serialize};

// The HTTP Gateway interfaces
// See https://github.com/nomeata/ic-http-lambda

#[derive(CandidType, Deserialize)]
struct HTTPQueryRequest {
    method: String,
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    uri: String,
    body: Vec<u8>,
}

#[derive(CandidType, Serialize)]
struct HTTPQueryResult {
    status: u16,
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    body: Vec<u8>,
    upgrade: bool,
}

// Some application logic

fn get_info() -> String {
    format!("This is a telgram bot on the Internet Computer!\nMy canister id: {}\nLocal time is {}ns.\nMy cycle balance is {}\nVisit my homepages:\nhttp://t.me/InternetComputerBot\nhttps://{}.ic.nomeata.de/\nhttps://github.com/nomeata/ic-telegram-bot.", id(), time(), canister_balance(), id())
}

static mut JOKE: Option<String> = None;

fn get_joke() -> String {
    unsafe {
        match &JOKE {
            None => "What does Mr. Williams reign over? His dom-minions!".to_string(),
            Some(joke) => joke.clone(),
        }
    }
}

fn set_joke(new_joke: String) {
    unsafe { JOKE = Some(new_joke) }
}

// Main entry points and dispatchers

#[update]
fn http_update(req: HTTPQueryRequest) -> HTTPQueryResult {
    dispatch(req)
}

#[query]
fn http_query(req: HTTPQueryRequest) -> HTTPQueryResult {
    dispatch(req)
}


fn dispatch(req: HTTPQueryRequest) -> HTTPQueryResult {
    let uri = req.uri.clone();
    match uri.strip_prefix("/webhook/") {
        Some(token) => handle_telegram(token, req),
        None => {
            if req.uri == "/" {
                index(req)
            } else {
                err404(req)
            }
        }
    }
}


// A common handlers

fn ok200() -> HTTPQueryResult {
    HTTPQueryResult {
        status: 200,
        headers: vec![(b"content-type".to_vec(), b"text/html".to_vec())],
        body: "Nothing to do".as_bytes().to_vec(),
        upgrade: false,
    }
}

fn index(_req: HTTPQueryRequest) -> HTTPQueryResult {
    HTTPQueryResult {
        status: 200,
        headers: vec![(b"content-type".to_vec(), b"text/plain".to_vec())],
        body: get_info().as_bytes().to_vec(),
        upgrade: false,
    }
}
fn err404(req: HTTPQueryRequest) -> HTTPQueryResult {
    HTTPQueryResult {
        status: 404,
        headers: vec![],
        body: format!(
            "Nothing found at {}\n(but still, you reached the internet computer!)",
            req.uri
        )
        .as_bytes()
        .to_vec(),
        upgrade: false,
    }
}

// Telegram boiler plate

use serde_json::Value;
use telegram_bot_raw::{MessageChat, MessageKind, SendMessage, Update, UpdateKind};

fn handle_telegram(_token: &str, req: HTTPQueryRequest) -> HTTPQueryResult {
    match serde_json::from_slice::<Update>(&req.body) {
        Err(err) => HTTPQueryResult {
            status: 500,
            headers: vec![],
            body: format!("{}", err).as_bytes().to_vec(),
            upgrade: false
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

fn send_message(chat: MessageChat, text: String) -> HTTPQueryResult {
    let m = SendMessage::new(chat, text);
    let mut value = serde_json::to_value(m).unwrap();
    add_method(&mut value, "sendMessage".to_string());
    HTTPQueryResult {
        status: 200,
        headers: vec![(b"content-type".to_vec(), b"application/json".to_vec())],
        body: serde_json::to_vec(&value).unwrap(),
        upgrade: false,
    }
}

// The actual handler

fn handle_message(chat: MessageChat, text: String) -> HTTPQueryResult {
    match text.as_str() {
        "/start" => send_message(
            chat,
            "Hello! I am a Telegram Bot in a canister. Try /joke, /info".to_string(),
        ),
        "/joke" => send_message(
            chat,
            format!(
                "{}\n(Got a better one? Tell me about it, with /telljoke …!)",
                get_joke()
            ),
        ),
        "/telljoke" => send_message(chat, "Put the joke after /telljoke!".to_string()),
        "/info" => send_message(chat, get_info()),
        _ => match text.strip_prefix("/telljoke ") {
            Some(joke) => {
                set_joke(joke.to_string());
                let mut resp = send_message(chat, "Ha! Ha! Duly noted.".to_string());
                resp.upgrade = true;
                resp
            }
            _ => send_message(chat, format!("What do you mean, {}?", text)),
        },
    }
}

