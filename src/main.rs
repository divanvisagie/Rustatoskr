use capabilities::{chat::chat::ChatCapability, Capability};

use std::env;
use teloxide::{prelude::*, types::ChatAction};

mod capabilities;
mod clients;
mod layers;

pub struct RequestMessage {
    text: String,
    username: String,
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
        RequestMessage {
            text: message.text().unwrap_or_default().to_string(),
            username: message.chat.username().unwrap_or_default().to_string(),
        }
    }
}

impl TelegramConverter {
    fn new() -> Self {
        TelegramConverter {}
    }
}

struct Handler {
    capabilities: Vec<Box<dyn Capability>>, // Needs to be trait Send as well
}

impl Handler {
    pub fn new() -> Self {
        Handler {
            capabilities: vec![Box::new(ChatCapability::new())],
        }
    }

    async fn handle_message(mut self, message: RequestMessage) -> ResponseMessage {
        let best = self.capabilities.iter_mut().reduce(|a, b| {
            if a.check(&message) > b.check(&message) {
                a
            } else {
                b
            }
        });

        let response = best.unwrap().execute(&message);
        response.await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let bc = TelegramConverter::new();

        let admin =
            env::var("TELEGRAM_ADMIN").expect("Missing TELEGRAM_ADMIN environment variable");
        if msg.chat.username().is_none() {
            bot.send_message(msg.chat.id, "You need to set a username to use this bot.")
                .await?;
            return Ok(());
        } else if msg.chat.username().unwrap() != admin {
            bot.send_message(
                msg.chat.id,
                format!("You need to contact @{} to use this bot.", admin),
            )
            .await?;
            return Ok(());
        }

        bot.send_chat_action(msg.chat.id, ChatAction::Typing)
            .await?;

        let hdlr = Handler::new();

        let req = bc.bot_type_to_request_message(&msg);
        let res = hdlr.handle_message(req).await;

        bot.send_message(msg.chat.id, res.text).await?;

        Ok(())
    })
    .await;

    Ok(())
}
