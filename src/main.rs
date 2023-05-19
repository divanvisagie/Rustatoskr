#![allow(deprecated)]
use layers::{memory::StoredMessage, selector};
use redis::{Client, Connection};
use teloxide::prelude::*;
use tokio::sync::Mutex;

use std::{env, sync::Arc};
use teloxide::types::{
    ChatAction, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, KeyboardButton,
    KeyboardMarkup, ParseMode, ReplyMarkup,
};

mod capabilities;
mod clients;
mod layers;

pub struct RequestMessage {
    text: String,
    username: String,
    context: Vec<StoredMessage>,
}

impl RequestMessage {
    pub fn new(text: String, username: String) -> Self {
        RequestMessage {
            text,
            username,
            context: Vec::new(),
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

struct Handler {
    gateway_layer: Box<dyn layers::Layer>,
}

impl Handler {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let selector_layer_box = Box::new(selector::SelectorLayer::new());
        let memory_layer = layers::memory::MemoryLayer::new(selector_layer_box, conn);
        let security_layer = layers::security::SecurityLayer::new(Box::new(memory_layer));
        Self {
            gateway_layer: Box::new(security_layer),
        }
    }

    async fn handle_message(mut self, message: &mut RequestMessage) -> ResponseMessage {
        self.gateway_layer.execute(message).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    // Command::repl(bot.clone(), answer).await;

    let wc = Arc::new(Mutex::new(get_redis_connection()));
    let keyboard_functions = vec!["Memory Dump", "Memory Clear"];

    let keyboard_row: Vec<KeyboardButton> = keyboard_functions
        .iter()
        .map(|title| KeyboardButton::new(title.to_string()))
        .collect();

    let keyboard = KeyboardMarkup::default()
        .append_row(keyboard_row)
        .resize_keyboard(true);

    let kbd = Arc::new(Mutex::new(keyboard.clone()));
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let conn = Arc::clone(&wc);
        let keyboard = Arc::clone(&kbd);

        async move {
            log::info!(">>> Received message: {}", msg.text().unwrap_or_default());
            let bc = TelegramConverter::new();
            let hdlr = Handler::new(conn);

            if msg.text().unwrap_or_default() == "/debug" {
                bot.send_message(msg.chat.id, "Welcome to the matrix!")
                    .parse_mode(ParseMode::Markdown)
                    .reply_markup(ReplyMarkup::Keyboard(keyboard.lock().await.clone()))
                    .await?;
                return Ok(());
            }

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
