use crate::{
    capabilities::Capability,
    clients::chatgpt::{GptClient, Role},
    RequestMessage, ResponseMessage,
};
use async_trait::async_trait;

pub struct ChatCapability {
    // fields omitted
    client: GptClient,
}

#[async_trait]
impl Capability for ChatCapability {
    fn check(&mut self, message: &RequestMessage) -> f32 {
        if message.text.len() > 0 {
            return 1.0;
        }
        0.5
    }

    async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage {
        self.client.add_message(Role::User, message.text.clone());
        let response = self.client.complete().await;
        self.client.add_message(Role::Assistant, response.clone());

        let msg = format!("{}", response);
        ResponseMessage {
            text: msg.to_string(),
        }
    }
}

impl ChatCapability {
    pub fn new() -> Self {
        ChatCapability {
            client: GptClient::new(),
        }
    }
}
