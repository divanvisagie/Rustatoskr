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
    next: Box<dyn Layer>,
}

fn save_to_redis(m_to_save: StoredMessage) {
    let url = env::var("REDIS_URL").expect("Missing REDIS_URL environment variable");
    let client = Client::open(url).expect("Failed to connect to Redis");

    let mut connection = client
        .get_connection()
        .expect("Failed to get Redis connection");

    // create a reference key
    let key = format!("messages:{}", m_to_save.username);

    log::info!("Saving message to Redis: {:?}", m_to_save);

    let value_for_key: String = connection.get(&key).unwrap_or("[]".to_string());
    log::info!("Value for key: {}", value_for_key);

    let mut current_messages: Vec<StoredMessage> =
        serde_json::from_str(&value_for_key).expect("Failed to deserialize messages from JSON");

    current_messages.push(m_to_save);

    if current_messages.len() > 15 {
        // keep only the last 15 messages
        current_messages.remove(0);
    }

    let json_message =
        serde_json::to_string(&current_messages).expect("Failed to serialize message to JSON");

    let _: () = connection
        .set(key, json_message)
        .expect("Failed to save message to Redis");
}

fn get_from_redis(username: String) -> Vec<StoredMessage> {
    let url = env::var("REDIS_URL").expect("Missing REDIS_URL environment variable");
    let client = Client::open(url).expect("Failed to connect to Redis");

    let mut connection = client
        .get_connection()
        .expect("Failed to get Redis connection");

    // create a reference key
    let key = format!("messages:{}", username);

    let value_for_key: String = connection.get(&key).unwrap_or("[]".to_string());
    log::info!("Value for key: {}", value_for_key);

    let current_messages: Vec<StoredMessage> =
        serde_json::from_str(&value_for_key).expect("Failed to deserialize messages from JSON");

    current_messages
}

#[async_trait]
impl Layer for MemoryLayer {
    async fn execute(&mut self, message: &mut RequestMessage) -> ResponseMessage {
        message.context = get_from_redis(message.username.clone());
        let res = self.next.execute(message).await;

        let user_message = StoredMessage {
            username: message.username.clone(),
            text: message.text.clone(),
            role: Role::User,
        };
        save_to_redis(user_message);

        let bot_message = StoredMessage {
            username: message.username.clone(),
            text: res.text.clone(),
            role: Role::Assistant,
        };
        log::info!("Saving bot message to Redis: {:?}", bot_message);
        save_to_redis(bot_message);

        res
    }
}

impl MemoryLayer {
    pub fn new(next: Box<dyn Layer>) -> Self {
        MemoryLayer { next }
    }
}
