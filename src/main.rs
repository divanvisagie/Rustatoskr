#![allow(deprecated)]
use layers::memory::StoredMessage;
use redis::Client;

use teloxide::prelude::*;
use tokio::sync::Mutex;

use std::{env, sync::Arc};
use teloxide::types::{
    ChatAction, InputFile, KeyboardButton, KeyboardMarkup, ParseMode, ReplyMarkup,
};

mod capabilities;
mod clients;
mod handler;
mod layers;
mod repositories;

pub struct RequestMessage {
    text: String,
    username: String,
    context: Vec<StoredMessage>,
    embedding: Vec<f32>,
}

impl RequestMessage {
    pub fn new(text: String, username: String) -> Self {
        RequestMessage {
            text,
            username,
            context: Vec::new(),
            embedding: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResponseMessage {
    text: String,

    /// If the response is a file, it will be sent to the user as a
    /// document and the text will be used as the filename
    bytes: Option<Vec<u8>>,

    /// Inline button options
    options: Option<Vec<String>>,
}

impl ResponseMessage {
    pub fn new(text: String) -> Self {
        ResponseMessage {
            text,
            bytes: None,
            options: None,
        }
    }

    pub fn new_with_bytes(text: String, bytes: Vec<u8>) -> Self {
        ResponseMessage {
            text,
            bytes: Some(bytes),
            options: None,
        }
    }

    pub fn new_with_options(text: String, options: Vec<String>) -> Self {
        ResponseMessage {
            text,
            bytes: None,
            options: Some(options),
        }
    }
}

struct TelegramConverter;

trait BotConverter<T> {
    fn bot_type_to_request_message(&self, bot_message: &T) -> RequestMessage;
}

impl BotConverter<Message> for TelegramConverter {
    fn bot_type_to_request_message(&self, message: &Message) -> RequestMessage {
        RequestMessage::new(
            message.text().unwrap_or_default().to_string(),
            message.chat.username().unwrap_or_default().to_string(),
        )
    }
}

impl TelegramConverter {
    fn new() -> Self {
        TelegramConverter {}
    }
}

fn get_redis_connection() -> redis::Connection {
    let url = env::var("REDIS_URL").expect("Missing REDIS_URL environment variable");
    let client = Client::open(url).expect("Failed to connect to Redis");

    client
        .get_connection()
        .expect("Failed to get Redis connection")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    let wc = Arc::new(Mutex::new(get_redis_connection()));

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let conn = Arc::clone(&wc);
        log::info!(
            "Got a message from: {}: {}",
            msg.chat.username().unwrap_or_default(),
            msg.text().unwrap_or_default()
        );

        async move {
            let bc = TelegramConverter::new();
            let hdlr = handler::Handler::new(conn);

            bot.send_chat_action(msg.chat.id, ChatAction::Typing)
                .await?;

            let mut req = bc.bot_type_to_request_message(&msg);
            let res = hdlr.handle_message(&mut req).await;

            if let Some(bytes) = res.bytes {
                bot.send_document(msg.chat.id, InputFile::memory(bytes).file_name(res.text))
                    .await?;

                return Ok(());
            }

            if let Some(options) = res.options {
                let keyboard_row: Vec<KeyboardButton> = options
                    .iter()
                    .map(|title| KeyboardButton::new(title.to_string()))
                    .collect();

                let keyboard = KeyboardMarkup::default()
                    .append_row(keyboard_row)
                    .resize_keyboard(true);

                bot.send_message(msg.chat.id, res.text)
                    .parse_mode(ParseMode::Markdown)
                    .reply_markup(ReplyMarkup::Keyboard(keyboard))
                    .await?;

                return Ok(());
            }

            bot.send_message(msg.chat.id, res.text)
                .parse_mode(ParseMode::Markdown)
                .await?;

            Ok(())
        }
    })
    .await;

    Ok(())
}
