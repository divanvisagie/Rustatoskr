#![allow(deprecated)]
use message_types::RequestMessage;
use redis::Client;

use teloxide::prelude::*;
use tokio::sync::Mutex;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use std::{env, sync::Arc};
use teloxide::types::{
    ChatAction, InputFile, KeyboardButton, KeyboardMarkup, ParseMode, ReplyMarkup,
};
use tokio::join;

mod capabilities;
mod clients;
mod handler;
mod layers;
mod message_types;
mod repositories;

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

pub async fn start_bot() {
    let bot = Bot::from_env();

    let wc = Arc::new(Mutex::new(get_redis_connection()));

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let conn = Arc::clone(&wc);
        log::info!(
            "{} sent the message: {}",
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

            match bot
                .send_message(msg.chat.id, res.text.clone())
                .parse_mode(ParseMode::Markdown)
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    bot.send_message(msg.chat.id, res.text.clone()).await?;
                    log::error!("Failed to send message: {}", e)
                }
            };

            Ok(())
        }
    })
    .await;
}

pub async fn start_server() {
    let _h = HttpServer::new(|| {
        App::new().service(hello)
        // .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await;
}

#[get("/health")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("All is good")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let http_server = start_server();
    let bot = start_bot();

    join!(http_server, bot);

    Ok(())
}
