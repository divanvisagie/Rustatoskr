use std::env;

use async_trait::async_trait;

use crate::{repositories::users::UserRepository, RequestMessage, ResponseMessage};

use super::Layer;
pub struct SecurityLayer {
    // fields omitted
    next: Box<dyn Layer>,
    admin: String,
    user_repository: Box<dyn UserRepository>,
}

#[async_trait]
impl Layer for SecurityLayer {
    async fn execute(&mut self, message: &mut RequestMessage) -> ResponseMessage {
        let users = self.user_repository.get_usernames().await;

        if users.contains(&message.username) || message.username == self.admin {
            self.next.execute(message).await
        } else {
            return ResponseMessage::new(format!(
                "You need to contact @{} to use this bot.",
                self.admin
            ));
        }
    }
}

impl SecurityLayer {
    pub fn new(next: Box<dyn Layer>, repo: Box<dyn UserRepository>) -> Self {
        let admin =
            env::var("TELEGRAM_ADMIN").expect("Missing TELEGRAM_ADMIN environment variable");
        SecurityLayer {
            next,
            admin,
            user_repository: repo,
        }
    }

    #[allow(dead_code)]
    pub fn with_admin(
        next: Box<dyn Layer>,
        user_repository: Box<dyn UserRepository>,
        admin: String,
    ) -> Self {
        SecurityLayer {
            next,
            admin,
            user_repository: user_repository,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use async_trait::async_trait;

    struct MockLayer {}
    #[async_trait]
    impl Layer for MockLayer {
        async fn execute(&mut self, message: &mut RequestMessage) -> ResponseMessage {
            ResponseMessage {
                text: format!("Hello, {}!", message.username),
                bytes: None,
                options: None,
            }
        }
    }

    #[tokio::test]
    async fn test_security_layer_not_allowed() {
        let mut layer = SecurityLayer::with_admin(Box::new(MockLayer {}), "valid_user".to_string());

        let mut message = RequestMessage {
            text: "Hello".to_string(),
            username: "invalid_user".to_string(),
            context: Vec::new(),
        };

        let response = layer.execute(&mut message).await;
        assert_eq!(
            response.text,
            "You need to contact @valid_user to use this bot."
        );
    }

    #[tokio::test]
    async fn test_security_layer_allowed() {
        let mut layer = SecurityLayer::with_admin(Box::new(MockLayer {}), "valid_user".to_string());

        let mut message = RequestMessage {
            text: "Hello".to_string(),
            username: "valid_user".to_string(),
            context: Vec::new(),
        };

        let response = layer.execute(&mut message).await;
        assert_eq!(response.text, "Hello, valid_user!");
    }
}
