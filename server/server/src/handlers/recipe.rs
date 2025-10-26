use crate::{
    auth::middleware::Auth,
    handlers::models::recipe::{
        ChangeVersionRequest, ChangeVersionResponse, GetRequest, GetResponse, UpdateRequest,
        UpdateResponse,
    },
    models::recipe::Recipes,
    state::AppState,
};

use axum::{Json, extract::State, http::StatusCode};

pub async fn get(
    State(state): State<AppState>,
    Auth(_user): Auth,
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
    .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = GetResponse { data: recipes };
    Ok(Json(response))
}

pub async fn update(
    State(_state): State<AppState>,
    Auth(_user): Auth,
    Json(payload): Json<UpdateRequest>,
) -> Result<Json<UpdateResponse>, StatusCode> {
    todo!()
}

pub async fn change_version(
    State(_state): State<AppState>,
    Auth(_user): Auth,
    Json(payload): Json<ChangeVersionRequest>,
) -> Result<Json<ChangeVersionResponse>, StatusCode> {
    todo!()
}
