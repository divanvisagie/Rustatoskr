use async_trait::async_trait;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::error::Error;

#[derive(Serialize)]
pub struct ChatRequest {
    pub role: String,
    pub content: String,
    pub hash: String,
}

#[derive(Serialize)]
pub struct SearchRequest {
    pub content: String,
}

#[derive(Deserialize, Clone)]
pub struct SearchResponse {
    pub role: String,
    pub content: String,
    pub hash: String,
    pub ranking: f32,
}

#[async_trait]
pub trait MunninClient: Send + Sync {
    async fn save(&self, role: String, content: String) -> Result<(), Box<dyn Error>>;
    async fn search(&self, query: String) -> Result<Vec<SearchResponse>, ()>;
}

pub struct MunninClientImpl {}

impl MunninClientImpl {
    pub fn new() -> Self {
        MunninClientImpl {}
    }
}

#[async_trait]
impl MunninClient for MunninClientImpl {
    async fn save(&self, role: String, content: String) -> Result<(), Box<dyn std::error::Error>> {
        let url = "http://heimdallr.local:8080/api/v1/chat";
        let client = reqwest::Client::new();

        // create a sha256 hash of the message
        let hash = Sha256::digest(content.as_bytes());

        let request_body = serde_json::to_string(&ChatRequest {
            role,
            content,
            hash: format!("{:x}", hash),
        });

        let request_body = match request_body {
            Ok(body) => body,
            Err(e) => panic!("Error serializing request body: {}", e),
        };

        let response = client
            .post(url)
            .body(request_body)
            .header("Content-Type", "application/json")
            .send()
            .await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Message sent successfully");
                } else {
                    println!("Failed to send message: {:?}", response);
                }
            }
            Err(e) => println!("Failed to send message: {}", e),
        }

        Ok(())
    }

    async fn search(&self, query: String) -> Result<Vec<SearchResponse>, ()> {
        let url = "http://heimdallr.local:8080/api/v1/chat";
        let client = reqwest::Client::new();

        let request_body = serde_json::to_string(&SearchRequest { content: query });

        let request_body = match request_body {
            Ok(body) => body,
            Err(e) => panic!("Error serializing request body: {}", e),
        };

        let response = client
            .post(url)
            .body(request_body)
            .header("Content-Type", "application/json")
            .send()
            .await;

        let response = match response {
            Ok(response) => response,
            Err(e) => {
                println!("Failed to send message: {}", e);
                return Err(());
            }
        };

        let response_body = response.json::<Vec<SearchResponse>>().await;
        let response_body = match response_body {
            Ok(body) => body,
            Err(e) => {
                println!("Failed to parse response: {}", e);
                return Err(());
            }
        };
        let var_name = Ok(response_body);
        var_name
    }
}
