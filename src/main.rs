#![allow(deprecated)]
use layers::{memory::StoredMessage, selector};
use redis::{Client, Connection};
use tokio::sync::Mutex;

use std::{env, sync::Arc};
use teloxide::{
    prelude::*,
    types::{ChatAction, ParseMode},
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

    let connection = client
        .get_connection()
        .expect("Failed to get Redis connection");

    connection
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
        let response = self.gateway_layer.execute(message).await;
        response
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
    let wc = Arc::new(Mutex::new(get_redis_connection()));
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let conn = Arc::clone(&wc);

        async move {
            let bc = TelegramConverter::new();
            let hdlr = Handler::new(conn);

            bot.send_chat_action(msg.chat.id, ChatAction::Typing)
                .await?;

            let mut req = bc.bot_type_to_request_message(&msg);
            let res = hdlr.handle_message(&mut req).await;

            bot.send_message(msg.chat.id, res.text)
                .parse_mode(ParseMode::Markdown)
                .await?;

            Ok(())
        }
    })
    .await;

    Ok(())
}
