use std::env;

use chatgpt::Role;
use teloxide::{prelude::*, types::ChatAction};

mod chatgpt;

struct RequestMessage {
    text: String,
}

struct ResponseMessage {
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
        }
    }
}

impl TelegramConverter {
    fn new() -> Self {
        TelegramConverter {}
    }
}

struct Handler {
    client: chatgpt::GptClient,
}

impl Handler {
    pub fn new() -> Self {
        Handler {
            client: chatgpt::GptClient::new(),
        }
    }

    async fn handle_message(mut self, message: RequestMessage) -> ResponseMessage {
        self.client.add_message(Role::User, message.text.clone());
        let response = self.client.complete().await;
        self.client.add_message(Role::Assistant, response.clone());

        let msg = format!("{}", response);
        ResponseMessage {
            text: msg.to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    async fn hdl<T>(bot: &Bot, msg: &Message, bc: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: BotConverter<Message>,
    {
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

        let hdlr = Handler::new();
        bot.send_chat_action(msg.chat.id, ChatAction::Typing)
            .await?;

        let req = bc.bot_type_to_request_message(msg);

        let resp = hdlr.handle_message(req).await;

        bot.send_message(msg.chat.id, resp.text).await?;

        Ok(())
    }

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let t_converter = TelegramConverter::new();
        hdl(&bot, &msg, t_converter).await.unwrap();

        Ok(())
    })
    .await;
}
