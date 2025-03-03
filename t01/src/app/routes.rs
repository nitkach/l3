use crate::{
    error::AppError,
    model::{Claims, CreatePostRequest, LoginRequest, RegisterForm},
    repository::Repository,
    utils::PasswordHash,
};
use askama::Template;
use auth::validate_jwt;
use axum::{
    extract::Path,
    response::{Html, Redirect},
    routing::delete,
    Extension, Form,
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use serde_json::json;
use tower_http::services::ServeDir;
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
    let router = Router::new()
        .route("/", get(get_index_page))
        .route("/register", get(get_register_page).post(register_user))
        .route("/login", get(get_login_page).post(login))
        .route("/posts", get(get_posts))
        .route("/posts/{post_id}", get(get_post));

    let secure_router = Router::new()
        .route("/posts", post(create_post))
        .route("/posts/{post_id}", delete(delete_post))
        .route("/posts/{post_id}/likes", post(like_post))
        .layer(axum::middleware::from_fn(validate_jwt));

    Router::new()
        .merge(router)
        .merge(secure_router)
        .nest_service("/static", ServeDir::new("templates"))
        .with_state(state)
}

#[derive(Debug, Template)]
#[template(path = "index.askama.html")]
struct IndexTemplate {}

#[axum::debug_handler]
async fn get_index_page(State(pool): State<Repository>) -> Result<impl IntoResponse, AppError> {
    let html = IndexTemplate {};

    Ok(askama_axum::into_response(&html))
}

#[derive(Debug, Template)]
#[template(path = "register.askama.html")]
struct RegisterTemplate {}

#[axum::debug_handler]
async fn get_register_page(State(pool): State<Repository>) -> Result<impl IntoResponse, AppError> {
    let html = RegisterTemplate {};

    Ok(askama_axum::into_response(&html))
}

#[derive(Debug, Template)]
#[template(path = "login.askama.html")]
struct LoginTemplate {}

#[axum::debug_handler]
async fn get_login_page(State(pool): State<Repository>) -> Result<impl IntoResponse, AppError> {
    let html = LoginTemplate {};

    Ok(askama_axum::into_response(&html))
}

#[axum::debug_handler]
async fn register_user(
    State(pool): State<Repository>,
    Json(payload): Json<RegisterForm>,
) -> Result<impl IntoResponse, AppError> {
    if payload.is_empty() {
        return Err(AppError::authenthication("Missing credentials".to_owned()));
    }

    let RegisterForm { username, password } = payload;
    let password_hash = PasswordHash::from_password(&password)?;

    pool.register_user(&username, password_hash).await?;

    Ok(Json(json!({ "message": "Registration successful" })))
}

async fn login(
    State(pool): State<Repository>,
    Json(login_req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if login_req.is_empty() {
        return Err(AppError::authenthication("Missing credentials".to_owned()));
    }

    let LoginRequest { username, password } = login_req;

    let Some(user) = pool.get_login_credentials(&username).await? else {
        return Err(AppError::not_found("User does not exists"));
    };

    if !user.password_hash.verify_password(&password)? {
        return Err(AppError::authenthication("Wrong credentials".to_owned()));
    }

    let user_id = user.user_id;
    let token = auth::create_access_token(user_id)?;

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
        return Err(AppError::not_found("Post does not exists"));
    };

    let json = json!({
        "id": post.id,
        "user_id": post.user_id,
        "content": post.content,
        "likes": post.likes,
    });

    Ok(Json(json))
}

async fn get_posts(
    State(pool): State<Repository>,
    Extension(_): Extension<Claims>,
    Path(post_id): Path<Ulid>,
) -> Result<impl IntoResponse, AppError> {
    let Some(post) = pool.get_post(post_id).await? else {
        return Err(AppError::not_found("Post does not exists"));
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
        Err(AppError::not_found("Post does not exists"))
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
        Err(AppError::not_found("Post does not exists"))
    }
}
