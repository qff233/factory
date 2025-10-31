use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{models::user::User, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = User::fetch_from_username(&state.db_pool, &payload.username)
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.verify_password(&payload.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // TODO 权限

    let token = crate::auth::token::add_token(&state.token_store, user.id).await;

    Ok(Json(LoginResponse { token, user }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(token) = crate::auth::middleware::extract_token_from_headers(&headers) {
        crate::auth::token::remove_token(&state.token_store, &token).await;
    }
    Ok(Json(serde_json::json!({"message": "登出成功"})))
}
