use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{models::user::User, state::AppState};

pub async fn token_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();

    let token = extract_token_from_headers(headers).ok_or(StatusCode::UNAUTHORIZED)?;

    let user_id = crate::auth::token::validate_token(&state.token_store, &token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user = User::from_id(&state.db_pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // TODO 权限

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?.to_str().ok()?;

    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        Some(auth_header.to_string())
    }
}
