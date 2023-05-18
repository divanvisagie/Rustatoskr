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

struct Handler;

impl Handler {
    pub fn new() -> Self {
        Handler {}
    }

    async fn handle_message(self, message: RequestMessage) -> ResponseMessage {
        let msg = format!("You said: {}", message.text);
        ResponseMessage {
            text: msg.to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    async fn hdl<T>(bot: &Bot, msg: &Message, bc: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: BotConverter<Message>,
    {
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
