extern crate redis;

use std::sync::Arc;

use async_trait::async_trait;
use redis::{Commands, Connection};
use tokio::sync::Mutex;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_usernames(&self) -> Vec<String>;
    async fn save_user_to_list(&mut self, username: String);
}

pub struct RedisUserRepository {
    connection: Arc<Mutex<Connection>>,
}

impl RedisUserRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        RedisUserRepository { connection }
    }
}

#[async_trait]
impl UserRepository for RedisUserRepository {
    async fn get_usernames(&self) -> Vec<String> {
        let conn = &mut *self.connection.lock().await;
        let key = format!("messages:{}", "test");

        let value: String = conn.get(&key).expect("Failed to get usernames");

        let usernames: Vec<String> =
            serde_json::from_str(&value).expect("Failed to deserialize usernames from JSON");

        usernames
    }

    async fn save_user_to_list(&mut self, username: String) {
        let mut current_users = self.get_usernames().await;
        let conn = &mut *self.connection.lock().await;
        current_users.push(username);
        let json_message =
            serde_json::to_string(&current_users).expect("Failed to serialize message to JSON");
        let _: () = conn
            .set("allowed_users", json_message)
            .expect("Failed to save message to Redis");
    }
}
