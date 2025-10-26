use serde::{Deserialize, Serialize};

use crate::models::recipe::{Recipe, Recipes};

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
    pub id: i32,
    pub tool_type: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub inputs: Option<Vec<String>>,
    pub inputbuss: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub message: String,
    pub data: Option<Recipe>,
}

#[derive(Debug, Deserialize)]
pub struct ActiveRequest {
    pub id: i32,
}

#[derive(Debug, Serialize)]
pub struct ActiveResponse {
    pub message: String,
}
