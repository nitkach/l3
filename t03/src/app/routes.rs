use crate::{
    dto::{GetMessages, PostJoinRoom, PostLeaveRoom, PostSendMessage},
    error::{AppError, ErrorKind},
    model::{Message, RoomKey, User},
    repository::Repository,
};
use axum::{
    debug_handler,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use serde_json::json;

pub(crate) fn initialize_router(state: Repository) -> Router {
    Router::new()
        .route("/join", post(join_room))
        .route("/leave", post(leave_room))
        .route("/send", post(send_message_to_room))
        .route("/messages", get(get_messages))
        .with_state(state)
}

#[debug_handler]
async fn join_room(
    State(pool): State<Repository>,
    Json(join_room): Json<PostJoinRoom>,
) -> Result<impl IntoResponse, AppError> {
    let room_key = RoomKey {
        id: join_room.room_key,
    };
    let user = User {
        username: join_room.user,
    };

    if !pool.add_user_to_room(room_key.clone(), user.clone()) {
        return Err(AppError::conflict(ErrorKind::User(format!(
            "user '{user}' already in room '{room_key}"
        ))));
    }

    Ok(StatusCode::OK)
}

async fn leave_room(
    State(pool): State<Repository>,
    Json(leave_room): Json<PostLeaveRoom>,
) -> Result<impl IntoResponse, AppError> {
    let room_key = RoomKey {
        id: leave_room.room_key,
    };
    let user = User {
        username: leave_room.user,
    };

    pool.remove_user_from_room(room_key, &user)?;

    Ok(())
}

async fn send_message_to_room(
    State(pool): State<Repository>,
    Json(send_message): Json<PostSendMessage>,
) -> Result<impl IntoResponse, AppError> {
    let room_key = RoomKey {
        id: send_message.room_key,
    };
    let user = User {
        username: send_message.user.clone(),
    };
    let message = Message {
        author: User {
            username: send_message.user,
        },
        content: send_message.message,
        timestamp: {
            let now = std::time::SystemTime::now();
            now.duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
        },
    };

    pool.send_message(room_key, &user, message)?;

    Ok(())
}

async fn get_messages(
    State(pool): State<Repository>,
    Json(get_messages): Json<GetMessages>,
) -> Result<impl IntoResponse, AppError> {
    let room_key = RoomKey {
        id: get_messages.room_key,
    };
    let user = User {
        username: get_messages.user,
    };

    let messages = pool.get_messages(room_key, &user)?;

    Ok(Json(json!(messages)))
}
