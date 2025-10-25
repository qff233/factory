use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AccountLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub username: String,
    pub role: String,
}
