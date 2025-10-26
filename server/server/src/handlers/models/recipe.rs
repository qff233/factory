use serde::{Deserialize, Serialize};

use crate::models::recipe::{self, Recipes};

#[derive(Debug, Deserialize)]
pub struct GetRequest {
    pub tool_type: Option<String>,
    pub recipe_name: Option<String>,
    pub page_count: Option<i32>,
    pub page_index: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct GetResponse {
    pub data: Recipes,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    id: i32,
    tool_type: Option<String>,
    name: Option<String>,
    status: Option<recipe::Status>,
    inputs: Option<Vec<String>>,
    inputbuss: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeVersionRequest {
    id: i32,
    version: String,
}

#[derive(Debug, Serialize)]
pub struct ChangeVersionResponse {
    message: String,
}
