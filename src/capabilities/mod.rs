use crate::{RequestMessage, ResponseMessage};
use async_trait::async_trait;

pub mod chat;

#[async_trait]
pub trait Capability: Send {
    fn check(&mut self, message: &RequestMessage) -> f32;
    async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage;
}
