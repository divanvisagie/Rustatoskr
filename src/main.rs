use teloxide::{dispatching::dialogue::GetChatId, prelude::*, types::ChatAction};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_chat_action(msg.chat.id, ChatAction::Typing)
            .await?;

        if let Some(text) = msg.text() {
            if text == "/dice" {
                bot.send_dice(msg.chat.id).await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    r#"Hi, I'm Rustatoskr, a bot designed because my creator 
                    was annoyed with Go interfaces which are objectively bad.
                    You can use /dice to throw a dice, or /help to get this message again."
                    "#
                    .trim_start_matches(" "),
                )
                .await?;
            }
        } else {
            bot.send_message(msg.chat.id, "I can't do that dave")
                .await?;
        }

        Ok(())
    })
    .await;
}
