use crate::{RequestMessage, ResponseMessage};
use async_trait::async_trait;

pub mod memory;
pub mod security;
pub mod selector;

#[async_trait]
pub trait Layer: Send {
    async fn execute(&mut self, message: &mut RequestMessage) -> ResponseMessage;
}
