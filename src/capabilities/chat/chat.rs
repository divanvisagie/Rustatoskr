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
        if !message.text.is_empty() {
            return 0.9;
        }
        0.5
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
        }
    }
}
