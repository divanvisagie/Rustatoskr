use std::env;

use async_trait::async_trait;

use crate::{RequestMessage, ResponseMessage};

use super::Layer;
pub struct SecurityLayer {
    // fields omitted
    next: Box<dyn Layer>,
    admin: String,
}

#[async_trait]
impl Layer for SecurityLayer {
    async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage {
        if message.username != self.admin {
            return ResponseMessage {
                text: format!("You need to contact @{} to use this bot.", self.admin),
            };
        } else {
            self.next.execute(message).await
        }
    }
}

impl SecurityLayer {
    #[allow(dead_code)]
    pub fn new(next: Box<dyn Layer>) -> Self {
        let admin =
            env::var("TELEGRAM_ADMIN").expect("Missing TELEGRAM_ADMIN environment variable");
        SecurityLayer { next, admin }
    }

    #[allow(dead_code)]
    pub fn with_admin(next: Box<dyn Layer>, admin: String) -> Self {
        SecurityLayer { next, admin }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use async_trait::async_trait;

    struct MockLayer {}
    #[async_trait]
    impl Layer for MockLayer {
        async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage {
            ResponseMessage {
                text: format!("Hello, {}!", message.username),
            }
        }
    }

    #[tokio::test]
    async fn test_security_layer_not_allowed() {
        let mut layer = SecurityLayer::with_admin(Box::new(MockLayer {}), "valid_user".to_string());

        let message = RequestMessage {
            text: "Hello".to_string(),
            username: "invalid_user".to_string(),
        };

        let response = layer.execute(&message).await;
        assert_eq!(
            response.text,
            "You need to contact @valid_user to use this bot."
        );
    }

    #[tokio::test]
    async fn test_security_layer_allowed() {
        let mut layer = SecurityLayer::with_admin(Box::new(MockLayer {}), "valid_user".to_string());

        let message = RequestMessage {
            text: "Hello".to_string(),
            username: "valid_user".to_string(),
        };

        let response = layer.execute(&message).await;
        assert_eq!(response.text, "Hello, valid_user!");
    }
}
