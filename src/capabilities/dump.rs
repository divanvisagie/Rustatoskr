use async_trait::async_trait;

use crate::{get_redis_connection, RequestMessage, ResponseMessage};

use super::Capability;

pub struct MemoryDumpCapability {}

#[async_trait]
impl Capability for MemoryDumpCapability {
    fn check(&mut self, message: &RequestMessage) -> f32 {
        if message.text == "Memory Dump" {
            return 1.0;
        }
        0.0
    }

    async fn execute(&mut self, _message: &RequestMessage) -> ResponseMessage {
        let mut redis_connection = get_redis_connection();
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("*")
            .query(&mut redis_connection)
            .unwrap();

        let mut result = String::new();
        for key in keys {
            let value: String = redis::cmd("GET")
                .arg(&key)
                .query(&mut redis_connection)
                .unwrap();
            result.push_str(&format!("{}: {}\n", key, value));
        }

        //convert to bytes
        let bytes = result.as_bytes().to_vec();

        ResponseMessage {
            text: "Memory Dump.csv".to_string(),
            bytes: Some(bytes),
        }
    }
}

impl MemoryDumpCapability {
    pub fn new() -> Self {
        MemoryDumpCapability {}
    }
}
