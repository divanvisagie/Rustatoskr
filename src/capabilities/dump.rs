use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

use crate::{
    get_redis_connection,
    layers::memory::{get_from_redis, StoredMessage},
    RequestMessage, ResponseMessage,
};

use super::Capability;

pub struct MemoryDumpCapability {}

#[async_trait]
impl Capability for MemoryDumpCapability {
    async fn check(&mut self, message: &RequestMessage) -> f32 {
        if message.text == "Memory Dump" {
            return 1.0;
        }
        0.0
    }

    async fn execute(&mut self, message: &RequestMessage) -> ResponseMessage {
        let mut redis_connection = get_redis_connection();

        let value: Vec<StoredMessage> =
            get_from_redis(&mut redis_connection, message.username.clone());

        //convert value to csv
        let mut csv_text = String::new();
        for message in value {
            csv_text.push_str(&format!("{}, {}\n", message.role, message.text));
        }

        //convert to bytes
        let bytes = csv_text.as_bytes().to_vec();

        //get unix timestamp as string
        let timestamp = get_current_timestamp();
        let filename = format!("Dump_{}_{}.csv", message.username, timestamp);

        ResponseMessage::new_with_bytes(filename, bytes)
    }
}

impl MemoryDumpCapability {
    pub fn new() -> Self {
        MemoryDumpCapability {}
    }
}

fn get_current_timestamp() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");

    timestamp.as_secs().to_string()
}
