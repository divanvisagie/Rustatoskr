use async_trait::async_trait;

use crate::{capabilities::Capability, RequestMessage, ResponseMessage};

use super::Layer;
struct SelectorLayer {
    // fields omitted
    capabilities: Vec<Box<dyn Capability>>,
}

#[async_trait]
impl Layer for SelectorLayer {
    async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage {
        let best = self.capabilities.iter_mut().reduce(|a, b| {
            if a.check(&message) > b.check(&message) {
                a
            } else {
                b
            }
        });
        best.unwrap().execute(&message).await
    }
}

impl SelectorLayer {
    #[allow(dead_code)]
    pub fn new(capabilities: Vec<Box<dyn Capability>>) -> Self {
        SelectorLayer { capabilities }
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
            ResponseMessage {
                text: format!("Hello, {}!", message.username),
            }
        }
    }

    #[tokio::test]
    async fn test_selector_layer() {
        let mut layer = SelectorLayer::new(vec![Box::new(MockCapability {})]);

        let message = RequestMessage {
            text: "Hello".to_string(),
            username: "test".to_string(),
        };
        let response = layer.execute(&message).await;
        assert_eq!(response.text, "Hello, test!");
    }
}
