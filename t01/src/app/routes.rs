use crate::{
    error::AppError,
    model::{Claims, CreatePostRequest, LoginRequest, RegisterRequest},
    repository::Repository,
};
use auth::validate_jwt;
use axum::{extract::Path, Extension};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bcrypt::verify;
use http::StatusCode;
use serde_json::json;
use ulid::Ulid;

mod auth;

/*
- `POST /register`: register a new user
- `POST /login`: log in a user
- `POST /posts`: create a new post
- `GET /posts/{post_id}`: get a post
- `DELETE /posts/{post_id}`: delete a post
- `POST /posts/{post_id}/likes`: like a post.
*/
pub(crate) fn initialize_router(state: Repository) -> Router {
    let secure_router = Router::new()
        .route("/posts", post(create_post))
        .route("/posts/:post_id", get(get_post).delete(delete_post))
        .route("/posts/:post_id/likes", post(like_post))
        .layer(axum::middleware::from_fn(validate_jwt));

    let router = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login));

    Router::new()
        .merge(router)
        .merge(secure_router)
        .with_state(state)
}

#[axum::debug_handler]
async fn register_user(
    State(pool): State<Repository>,
    Json(register_req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    if register_req.is_empty() {
        return Err(AppError::Authenthication("Missing credentials".to_owned()));
    }

    pool.register_user(register_req).await?;

    Ok(StatusCode::CREATED)
}

async fn login(
    State(pool): State<Repository>,
    Json(login_req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if login_req.is_empty() {
        return Err(AppError::Authenthication("Missing credentials".to_owned()));
    }

    let Some(user) = pool.login_user(&login_req).await? else {
        return Err(AppError::NotFound("User does not exists".to_owned()));
    };

    if !verify(login_req.password, &user.password_hash).map_err(anyhow::Error::from)? {
        return Err(AppError::Authenthication("Wrong credentials".to_owned()));
    };

    let token = auth::create_access_token(user.id)?;
    let user_id = user.id;

    let json = json!({
        "token": token,
        "user_id": user_id,
    });

    Ok(Json(json))
}

#[axum::debug_handler]
async fn create_post(
    State(pool): State<Repository>,
    Extension(claims): Extension<Claims>,
    Json(post): Json<CreatePostRequest>,
) -> Result<impl IntoResponse, AppError> {
    pool.create_post(post, claims).await?;

    Ok(StatusCode::CREATED)
}

async fn get_post(
    State(pool): State<Repository>,
    Extension(_): Extension<Claims>,
    Path(post_id): Path<Ulid>,
) -> Result<impl IntoResponse, AppError> {
    let Some(post) = pool.get_post(post_id).await? else {
        return Err(AppError::NotFound("Post does not exists".to_owned()));
    };

    let json = json!({
        "id": post.id,
        "user_id": post.user_id,
        "content": post.content,
        "likes": post.likes,
    });

    Ok(Json(json))
}

async fn delete_post(
    State(pool): State<Repository>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<Ulid>,
) -> Result<impl IntoResponse, AppError> {
    if pool.delete_post(post_id, claims).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Post does not exists".to_owned()))
    }
}

async fn like_post(
    State(pool): State<Repository>,
    Extension(_): Extension<Claims>,
    Path(post_id): Path<Ulid>,
) -> Result<impl IntoResponse, AppError> {
    if pool.like_post(post_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Post does not exists".to_owned()))
    }
}
