use crate::{RequestMessage, ResponseMessage};
use async_trait::async_trait;

pub mod chat;
pub mod dump;
pub mod privacy;
pub mod summarize;

#[async_trait]
pub trait Capability: Send {
    async fn check(&mut self, message: &RequestMessage) -> f32;
    async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage;
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot_product / (magnitude_a * magnitude_b)
}
