extern crate redis;

use std::sync::Arc;

use async_trait::async_trait;
use redis::{Commands, Connection};
use std::env;
use tokio::sync::Mutex;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_usernames(&mut self) -> Vec<String>;
}

pub struct FsUserRepository {}

impl FsUserRepository {
    pub fn new() -> Self {
        FsUserRepository {}
    }
}

async fn create_allowed_users(conn: &mut Connection) {
    let _: () = conn
        .set("allowed_users", "[]")
        .expect("Failed to save message to Redis");
}
#[async_trait]
impl UserRepository for FsUserRepository {
    async fn get_usernames(&mut self) -> Vec<String> {
        // get admin user from env
        let admin_user = env::var("TELEGRAM_ADMIN").expect("TELEGRAM_ADMIN must be set");
        return vec![admin_user];
    }
}
