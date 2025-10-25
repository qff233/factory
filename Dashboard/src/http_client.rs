use std::cell::RefCell;

use reqwest::header::{HeaderMap, HeaderValue};
use tokio::sync::mpsc;

use crate::{
    constant,
    model::account::{Account, AccountLogin},
};

#[derive(Debug)]
pub enum Error {
    #[allow(dead_code)]
    Reqwest(reqwest::Error),
    InvalidAccount,
    Internal,
}

pub struct HttpClient {
    headers: RefCell<HeaderMap>,
    timeout_sender: mpsc::Sender<()>,
}

impl HttpClient {
    pub fn new(timeout_callback: impl Fn() -> () + 'static + std::marker::Send) -> Self {
        let (sender, mut receiver) = mpsc::channel(5);

        tokio::spawn(async move {
            loop {
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(constant::TIMER_INTERVAL),
                    async {
                        receiver.recv().await;
                    },
                )
                .await
                {
                    Ok(_) => {}
                    Err(_) => {
                        timeout_callback();
                        println!("timeout! Auto logout!");
                    }
                }
            }
        });

        Self {
            headers: RefCell::new(HeaderMap::new()),
            timeout_sender: sender,
        }
    }

    pub async fn login(&self, account: AccountLogin) -> Result<Account, Error> {
        let json: serde_json::Value = reqwest::Client::new()
            .post(format!("{}/login", constant::URL))
            .json(&account)
            .send()
            .await
            .map_err(|e| Error::Reqwest(e))?
            .json()
            .await
            .map_err(|e| Error::Reqwest(e))?;

        let auth = HeaderValue::from_str(
            format!(
                "Bearer {}",
                json.get("token")
                    .ok_or(Error::InvalidAccount)?
                    .as_str()
                    .ok_or(Error::InvalidAccount)?
                    .to_string()
            )
            .as_str(),
        )
        .map_err(|_e| Error::Internal)?;
        self.headers.borrow_mut().insert("Authorization", auth);

        let user_json = json.get("user").ok_or(Error::Internal)?;

        let username = account.username;
        let account = Account {
            username,
            role: user_json
                .get("role")
                .ok_or(Error::Internal)?
                .as_str()
                .ok_or(Error::Internal)?
                .to_string(),
        };

        Ok(account)
    }

    pub async fn logout(&self) -> Result<String, Error> {
        self.timeout_sender.send(()).await.unwrap();
        let json: serde_json::Value = reqwest::Client::new()
            .post(format!("{}/logout", constant::URL))
            .headers(self.headers.borrow().clone())
            .send()
            .await
            .map_err(|e| Error::Reqwest(e))?
            .json()
            .await
            .map_err(|e| Error::Reqwest(e))?;
        Ok(json
            .get("message")
            .ok_or(Error::Internal)?
            .as_str()
            .ok_or(Error::Internal)?
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{http_client::HttpClient, model::account::AccountLogin};

    #[tokio::test]
    async fn login() {
        let http = HttpClient::new(|| {});

        let account = AccountLogin {
            username: "qff233".to_string(),
            password: "test_hash".to_string(),
        };
        let account = http.login(account).await.unwrap();
        println!("{:#?}", account);

        let result = http.logout().await.unwrap();
        println!("{}", result);
    }
}
