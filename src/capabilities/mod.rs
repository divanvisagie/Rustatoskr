use crate::{RequestMessage, ResponseMessage};

pub mod chat;

pub trait Capability: Send {
    fn Check(&mut self, message: RequestMessage) -> f32;
    fn Execute(&mut self, message: RequestMessage) -> ResponseMessage;
}
