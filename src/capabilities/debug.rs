use async_trait::async_trait;
use std::any::type_name;

use crate::{
    capabilities::cosine_similarity, clients::embeddings::EmbeddingsClient, RequestMessage,
    ResponseMessage,
};

use super::Capability;

struct DebugCapability {
    description: String,
}

#[async_trait]
impl Capability for DebugCapability {
    async fn check(&mut self, message: &RequestMessage) -> f32 {
        let cl = EmbeddingsClient::new();

        let description_embedding = cl.get_embeddings(self.description.clone()).await.unwrap();

        let similarity = cosine_similarity(
            message.embedding.as_slice(),
            description_embedding.as_slice(),
        );
        log::info!(
            "{} similarity: {}",
            type_name::<DebugCapability>(),
            similarity.clone()
        );
        similarity
    }

    async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage {
        ResponseMessage::new(format!("Debug: {}", message.text))
    }
}
