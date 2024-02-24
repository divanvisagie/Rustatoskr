extern crate redis;
use redis::{Commands, Connection};

use serde::{Deserialize, Serialize};

use async_trait::async_trait;

use crate::{
    clients::{
        chatgpt::Role,
        muninn::{MunninClient, MunninClientImpl},
    },
    message_types::ResponseMessage,
    RequestMessage,
};

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
        let munnin_client = MunninClientImpl::new();

        message.context = vec![];
        let res = self.next.execute(message).await;

        munnin_client
            .save("user".to_string(), message.text.clone())
            .await
            .unwrap();

        munnin_client
            .save("assistant".to_string(), res.text.clone())
            .await
            .unwrap();

        res
    }
}

impl MemoryLayer {
    pub fn new(next: Box<dyn Layer>) -> Self {
        MemoryLayer { next }
    }
}
