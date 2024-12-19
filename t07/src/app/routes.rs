use super::Repository;
use crate::{
    error::AppError,
    model::{Event, Subscription},
};
use anyhow::Result;
use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use log::info;
use serde_json::json;

#[debug_handler]
pub(crate) async fn create_event(
    State(mut state): State<Repository>,
    Json(event): Json<Event>,
) -> Result<impl IntoResponse, AppError> {
    let ulid = state.add_event(&event).await?;
    info!("Added event with ulid: {ulid}");

    state.notify_subscribers(&event).await?;
    info!("Subscribers notified");

    Ok(StatusCode::OK)
}

#[debug_handler]
pub(crate) async fn subscribe(
    State(mut state): State<Repository>,
    Json(subscription): Json<Subscription>,
) -> Result<impl IntoResponse, AppError> {
    state.add_subscription(&subscription).await?;
    Ok(())
}

#[debug_handler]
pub(crate) async fn get_user_events(
    State(mut state): State<Repository>,
    Path(user_id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let events = state.get_user_events(user_id).await?;

    let json = json!({"user_id":user_id,"events":events,});
    Ok(Json(json))
}
