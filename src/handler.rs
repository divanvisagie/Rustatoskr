use std::sync::Arc;

use redis::Connection;
use tokio::sync::Mutex;

use crate::{
    layers::{
        embedding::EmbeddingLayer, memory::MemoryLayer, security::SecurityLayer,
        selector::SelectorLayer, Layer,
    },
    repositories::users::RedisUserRepository,
    RequestMessage, ResponseMessage,
};

pub struct Handler {
    gateway_layer: Box<dyn Layer>,
}

impl Handler {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let selector_layer = SelectorLayer::new();
        let embedding_layer = EmbeddingLayer::new(Box::new(selector_layer));
        let memory_layer = MemoryLayer::new(Box::new(embedding_layer), Arc::clone(&conn));

        let user_repository = RedisUserRepository::new(Arc::clone(&conn));
        let security_layer = SecurityLayer::new(Box::new(memory_layer), Box::new(user_repository));
        Self {
            gateway_layer: Box::new(security_layer),
        }
    }

    pub async fn handle_message(mut self, message: &mut RequestMessage) -> ResponseMessage {
        self.gateway_layer.execute(message).await
    }
}
