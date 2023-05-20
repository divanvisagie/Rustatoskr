use crate::{
    capabilities::{cosine_similarity, Capability},
    clients::{
        chatgpt::{GptClient, Role},
        embeddings::EmbeddingsClient,
    },
    RequestMessage, ResponseMessage,
};
use async_trait::async_trait;
use std::any::type_name;

#[derive(Debug)]
pub struct ChatCapability {
    // fields omitted
    client: GptClient,
    description: String,
}

#[async_trait]
impl Capability for ChatCapability {
    async fn check(&mut self, message: &RequestMessage) -> f32 {
        let cl = EmbeddingsClient::new();

        let description_embedding = cl.get_embeddings(self.description.clone()).await.unwrap();

        let similarity = cosine_similarity(
            message.embedding.as_slice(),
            description_embedding.as_slice(),
        );
        log::info!(
            "{} similarity: {}",
            type_name::<ChatCapability>(),
            similarity.clone()
        );
        similarity
    }

    async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage {
        message.context.iter().for_each(|m| {
            self.client.add_message(m.role.clone(), m.text.clone());
        });

        self.client.add_message(Role::User, message.text.clone());
        let response = self.client.complete().await;

        ResponseMessage::new(response)
    }
}

impl ChatCapability {
    pub fn new() -> Self {
        ChatCapability {
            client: GptClient::new(),
            description: "General questions".to_string(),
        }
    }
}
