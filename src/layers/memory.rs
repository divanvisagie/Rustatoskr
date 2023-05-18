extern crate redis;
use redis::{Client, Commands};

use serde::{Deserialize, Serialize};
use std::env;

use async_trait::async_trait;

use crate::{clients::chatgpt::Role, RequestMessage, ResponseMessage};

use super::Layer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredMessage {
    pub username: String,
    pub text: String,
    pub role: Role,
}

pub struct MemoryLayer {
    stored_messages: Vec<StoredMessage>,
    next: Box<dyn Layer>,
}

fn save_to_redis(message: StoredMessage) {
    let url = env::var("REDIS_URL").expect("Missing REDIS_URL environment variable");
    let client = Client::open(url).expect("Failed to connect to Redis");

    let mut connection = client
        .get_connection()
        .expect("Failed to get Redis connection");

    let to_save = vec![&message];
    let json_message =
        serde_json::to_string(&to_save).expect("Failed to serialize message to JSON");

    let key = format!("messages:{}", message.username);
    let _: () = connection
        .set(key, json_message)
        .expect("Failed to save message to Redis");
}

#[async_trait]
impl Layer for MemoryLayer {
    async fn execute(&mut self, message: &mut RequestMessage) -> ResponseMessage {
        message.context = self.stored_messages.clone();
        let res = self.next.execute(message).await;

        let user_message = StoredMessage {
            username: message.username.clone(),
            text: message.text.clone(),
            role: Role::User,
        };
        self.stored_messages.push(user_message.clone());
        save_to_redis(user_message);

        let bot_message = StoredMessage {
            username: "bot".to_string(),
            text: res.text.clone(),
            role: Role::Assistant,
        };
        self.stored_messages.push(bot_message);

        res
    }
}

impl MemoryLayer {
    pub fn new(next: Box<dyn Layer>) -> Self {
        MemoryLayer {
            stored_messages: Vec::new(),
            next,
        }
    }
}
