use crate::repository::Repository;
use axum::{
    routing::{get, post},
    Router,
};
use routes::{create_event, get_user_events, subscribe};

mod error;
mod routes;

pub(crate) fn initialize_router(shared_state: Repository) -> Router {
    Router::new()
        .route("/events", post(create_event))
        .route("/subscribe", post(subscribe))
        .route("/events/:user_id", get(get_user_events))
        .with_state(shared_state)
}
