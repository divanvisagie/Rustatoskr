use super::Layer;
use crate::capabilities::chat::chat::ChatCapability;
use crate::capabilities::dump::MemoryDumpCapability;
use crate::capabilities::privacy::PrivacyCapability;
use crate::capabilities::summarize::SummaryCapability;
use crate::{capabilities::Capability, RequestMessage, ResponseMessage};
use async_trait::async_trait;
pub struct SelectorLayer {
    // fields omitted
    capabilities: Vec<Box<dyn Capability>>,
}

#[async_trait]
impl Layer for SelectorLayer {
    async fn execute(&mut self, message: &mut RequestMessage) -> ResponseMessage {
        let mut best: Option<&mut Box<dyn Capability>> = None;
        let mut best_score = 0.0;

        for capability in &mut self.capabilities {
            let score = capability.check(message).await;
            if score > best_score {
                best_score = score;
                best = Some(capability);
            }
        }
        best.unwrap().execute(message).await
    }
}

impl SelectorLayer {
    pub fn new() -> Self {
        SelectorLayer {
            capabilities: vec![
                Box::new(PrivacyCapability::new()),
                Box::new(MemoryDumpCapability::new()),
                Box::new(ChatCapability::new()),
                Box::new(SummaryCapability::new()),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockCapability {}
    #[async_trait]
    impl Capability for MockCapability {
        fn check(&mut self, message: &RequestMessage) -> f32 {
            if message.text == "hi" {
                1.0
            } else {
                0.0
            }
        }
        async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage {
            ResponseMessage::new(format!("Hello, {}!", message.username))
        }
    }

    #[tokio::test]
    async fn test_selector_layer() {
        let mut layer = SelectorLayer {
            capabilities: vec![Box::new(MockCapability {})],
        };

        let mut message = RequestMessage {
            text: "Hello".to_string(),
            username: "test".to_string(),
            context: Vec::new(),
        };
        let response = layer.execute(&mut message).await;
        assert_eq!(response.text, "Hello, test!");
    }
}
