use super::{error::AppError, Repository};
use crate::model::Event;
use anyhow::Result;
use axum::{debug_handler, extract::State, response::IntoResponse, Json};
use std::sync::Arc;

#[debug_handler]
pub(crate) async fn create_event(
    State(state): State<Repository>,
    Json(event): Json<Event>,
) -> Result<impl IntoResponse, AppError> {
    
    Ok(())
}

#[debug_handler]
pub(crate) async fn subscribe() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

#[debug_handler]
pub(crate) async fn get_user_events() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
