use layers::selector;

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
    gateway_layer: Box<dyn layers::Layer>,
}

impl Handler {
    pub fn new() -> Self {
        let selector_layer_box = Box::new(selector::SelectorLayer::new());
        let security_layer = layers::security::SecurityLayer::new(selector_layer_box);
        Handler {
            gateway_layer: Box::new(security_layer),
        }
    }

    async fn handle_message(mut self, message: RequestMessage) -> ResponseMessage {
        let response = self.gateway_layer.execute(&message).await;
        response
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
