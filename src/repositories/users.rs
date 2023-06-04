extern crate redis;

use std::sync::Arc;

use async_trait::async_trait;
use redis::{Commands, Connection};
use tokio::sync::Mutex;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_usernames(&mut self) -> Vec<String>;
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

async fn create_allowed_users(conn: &mut Connection) {
    let _: () = conn
        .set("allowed_users", "[]")
        .expect("Failed to save message to Redis");
}
#[async_trait]
impl UserRepository for RedisUserRepository {
    async fn get_usernames(&mut self) -> Vec<String> {
        let conn = &mut self.connection.lock().await;

        let value = match conn.get("allowed_users") {
            Ok(value) => value,
            Err(err) => {
                create_allowed_users(conn).await;
                log::error!("Failed to get message from Redis: {}", err);
                "[]".to_string()
            }
        };

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
