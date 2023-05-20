use async_trait::async_trait;
use std::any::type_name;

use crate::{
    capabilities::cosine_similarity, clients::embeddings::EmbeddingsClient, RequestMessage,
    ResponseMessage,
};

use super::Capability;

pub struct DebugCapability {
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
        log::info!("{} similarity: {}", type_name::<Self>(), similarity.clone());
        similarity
    }

    async fn execute(&mut self, _message: &RequestMessage) -> ResponseMessage {
        let keyboard_functions = vec!["Memory Dump".to_string(), "Memory Clear".to_string()];
        ResponseMessage::new_with_options(
            "I've sent you some debug options, you should see the buttons below.".to_string(),
            keyboard_functions,
        )
    }
}

impl DebugCapability {
    pub fn new() -> Self {
        let description = "Debugging capability".to_string();
        DebugCapability { description }
    }
}
