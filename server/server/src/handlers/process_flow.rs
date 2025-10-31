use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    models::process_flow::{ProcessFlow, ProcessFlowNames},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct GetNamesRequest {
    limit: i32,
    offset: i32,
}

#[derive(Debug, Serialize)]
pub struct GetNamesResponse {
    data: ProcessFlowNames,
}

pub async fn get_names(
    State(state): State<AppState>,
    Json(payload): Json<GetNamesRequest>,
) -> Result<Json<GetNamesResponse>, StatusCode> {
    let data = ProcessFlowNames::fetch_all(&state.db_pool, payload.limit, payload.offset)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(GetNamesResponse { data }))
}

#[derive(Debug, Deserialize)]
pub struct GetFlowFromNameRequest {
    name: String,
}

#[derive(Debug, Serialize)]
pub struct GetFlowFromNameResponse {
    data: ProcessFlow,
}

pub async fn get(
    State(state): State<AppState>,
    Json(payload): Json<GetFlowFromNameRequest>,
) -> Result<Json<GetFlowFromNameResponse>, StatusCode> {
    let data = ProcessFlow::fetch_from_name(&state.db_pool, &payload.name)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(GetFlowFromNameResponse { data }))
}
