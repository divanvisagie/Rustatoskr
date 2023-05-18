use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    is_bot: bool,
    first_name: String,
    last_name: String,
    username: String,
    language_code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chat {
    id: u64,
    first_name: String,
    last_name: String,
    username: String,
    type_: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    message_id: u64,
    from: User,
    date: u64,
    chat: Chat,
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Update {
    update_id: u64,
    message: Message,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UpdatesResponse {
    ok: bool,
    result: Vec<Update>,
}

pub const URL: &str = "https://api.telegram.org";

pub struct TelegramPollingClient {
    token: String,
}

impl TelegramPollingClient {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    async fn get_updates(self) -> UpdatesResponse {
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/getUpdates", self.token);

        let response = client.get(url).send().await;

        let response = match response {
            Ok(response) => response.text().await,
            Err(e) => panic!("Error: {}", e),
        };
        let string = response.unwrap();
        let value: UpdatesResponse = serde_json::from_str(string.as_str()).unwrap();
        value
    }
}

fn method_url(base: reqwest::Url, token: &str, method_name: &str) -> reqwest::Url {
    base.join(&format!("/bot{token}/{method_name}"))
        .expect("failed to format url")
}

fn make_request_url(base: reqwest::Url, token: &str, method_name: &str) -> reqwest::Url {
    base.join(&format!("/bot{token}/{method_name}"))
        .expect("failed to format url")
}

#[cfg(test)]
mod tests {
    use std::env;
    use tokio::test;

    use crate::clients::telegram::{method_url, TelegramPollingClient, URL};

    #[test]
    async fn method_url_test() {
        let url = method_url(
            reqwest::Url::parse(URL).unwrap(),
            "555362388:AAAA-g0gYncWnm5IyfZlpPRqRRv6kNAGlao",
            "methodName",
        );

        assert_eq!(
            url.as_str(),
            "https://api.telegram.org/bot555362388:AAAA-g0gYncWnm5IyfZlpPRqRRv6kNAGlao/methodName"
        );
    }

    #[test]
    async fn test_get_updates() {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("Missing TELEGRAM_BOT_TOKEN");
        let client = TelegramPollingClient::new(bot_token);

        let updates = client.get_updates().await;

        assert!(updates.ok)
    }
}
