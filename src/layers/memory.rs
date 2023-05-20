extern crate redis;
use redis::{Commands, Connection};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;

use crate::{clients::chatgpt::Role, message_types::ResponseMessage, RequestMessage};

use super::Layer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredMessage {
    pub username: String,
    pub text: String,
    pub role: Role,
}

pub struct MemoryLayer {
    next: Box<dyn Layer>,
    connection: Arc<Mutex<Connection>>,
}

fn save_to_redis(connection: &mut Connection, m_to_save: StoredMessage) {
    // create a reference key
    let key = format!("messages:{}", m_to_save.username);

    let value_for_key: String = connection.get(&key).unwrap_or("[]".to_string());

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

pub fn get_from_redis(connection: &mut Connection, username: String) -> Vec<StoredMessage> {
    // create a reference key
    let key = format!("messages:{}", username);

    let value_for_key: String = connection.get(&key).unwrap_or("[]".to_string());

    let current_messages: Vec<StoredMessage> =
        serde_json::from_str(&value_for_key).expect("Failed to deserialize messages from JSON");

    current_messages
}

#[async_trait]
impl Layer for MemoryLayer {
    async fn execute(&mut self, message: &mut RequestMessage) -> ResponseMessage {
        let conn = &mut *self.connection.lock().await;
        message.context = get_from_redis(conn, message.username.clone());
        let res = self.next.execute(message).await;

        let user_message = StoredMessage {
            username: message.username.clone(),
            text: message.text.clone(),
            role: Role::User,
        };
        save_to_redis(conn, user_message);

        let bot_message = StoredMessage {
            username: message.username.clone(),
            text: res.text.clone(),
            role: Role::Assistant,
        };
        save_to_redis(conn, bot_message);

        res
    }
}

impl MemoryLayer {
    pub fn new(next: Box<dyn Layer>, connection: Arc<Mutex<Connection>>) -> Self {
        MemoryLayer { next, connection }
    }
}
