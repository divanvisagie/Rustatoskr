use async_trait::async_trait;
use std::env;

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

#[async_trait]
impl UserRepository for FsUserRepository {
    async fn get_usernames(&mut self) -> Vec<String> {
        // get admin user from env
        let admin_user = env::var("TELEGRAM_ADMIN").expect("TELEGRAM_ADMIN must be set");
        return vec![admin_user];
    }
}
