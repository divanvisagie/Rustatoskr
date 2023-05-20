use async_trait::async_trait;

use crate::{
    capabilities::cosine_similarity, clients::embeddings::EmbeddingsClient,
    message_types::ResponseMessage, RequestMessage,
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

        cosine_similarity(
            message.embedding.as_slice(),
            description_embedding.as_slice(),
        )
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
