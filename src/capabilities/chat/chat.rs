use crate::{
    capabilities::Capability,
    clients::chatgpt::{GptClient, Role},
    RequestMessage, ResponseMessage,
};

pub struct ChatCapability {
    // fields omitted
    client: GptClient,
}

impl Capability for ChatCapability {
    fn Check(&mut self, message: RequestMessage) -> f32 {
        1.0
    }

    fn Execute(&mut self, message: RequestMessage) -> ResponseMessage {
        ResponseMessage {
            text: "Hello, world!".to_string(),
        }
    }
}

impl ChatCapability {
    pub fn new() -> Self {
        ChatCapability {
            client: GptClient::new(),
        }
    }

    async fn handle_message(mut self, message: RequestMessage) -> ResponseMessage {
        self.client.add_message(Role::User, message.text.clone());
        let response = self.client.complete().await;
        self.client.add_message(Role::Assistant, response.clone());

        let msg = format!("{}", response);
        ResponseMessage {
            text: msg.to_string(),
        }
    }
}
