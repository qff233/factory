use crate::{
    auth::middleware::Auth,
    handlers::models::recipe::{
        ActiveRequest, ActiveResponse, GetRequest, GetResponse, UpdateRequest, UpdateResponse,
    },
    models::recipe::{Recipe, Recipes},
    state::AppState,
};

use axum::{Json, extract::State, http::StatusCode};
use tracing::error;

pub async fn get(
    State(state): State<AppState>,
    Auth(user): Auth,
    Json(payload): Json<GetRequest>,
) -> Result<Json<GetResponse>, StatusCode> {
    let pool = &state.db_pool;

    let recipes = Recipes::from_tool_type_and_name(
        pool,
        payload.tool_type.as_ref().map(|s| s.as_str()),
        payload.recipe_name.as_ref().map(|s| s.as_str()),
        payload.page_count,
        payload.page_index,
    )
    .await
    .map_err(|e| {
        error!("{}: {:#?}", user.username, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let response = GetResponse { data: recipes };
    Ok(Json(response))
}

pub async fn update(
    State(state): State<AppState>,
    Auth(user): Auth,
    Json(payload): Json<UpdateRequest>,
) -> Result<Json<UpdateResponse>, StatusCode> {
    let data = Recipe::update(
        &state.db_pool,
        payload.id,
        payload.tool_type.as_ref().map(|s| s.as_str()),
        payload.name.as_ref().map(|s| s.as_str()),
        payload.version.as_ref().map(|s| s.as_str()),
        payload.inputs.as_ref(),
        payload.inputbuss.as_ref(),
    )
    .await
    .map_err(|e| {
        error!("{}: {:#?}", user.username, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(UpdateResponse {
        message: "更新成功！".to_string(),
        data: Some(data),
    }))
}

pub async fn active(
    State(state): State<AppState>,
    Auth(user): Auth,
    Json(payload): Json<ActiveRequest>,
) -> Result<Json<ActiveResponse>, StatusCode> {
    let message = Recipe::active(&state.db_pool, payload.id)
        .await
        .map_err(|e| {
            error!("{}: {:#?}", user.username, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(ActiveResponse { message }))
}
