use crate::{
    error::AppError,
    model::{Claims, CreatePostRequest, LoginRequest, RegisterRequest},
    repository::{PostDeleteResult, Repository},
    utils::PasswordHash,
};
use askama::Template;
use auth::validate_jwt;
use axum::{extract::Path, Extension};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use log::{info, warn};
use serde_json::json;
use tower_http::services::ServeDir;

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
        .route("/", get(get_page_index))
        .route("/register", get(get_page_registration))
        .route("/api/register", post(register_user))
        .route("/login", get(get_page_login))
        .route("/api/login", post(login_user))
        .route("/posts", get(get_page_posts))
        .route("/posts/:post_id", get(get_page_post))
        .route("/users/:user_id", get(get_page_user));

    let secure_router = Router::new()
        .route("/api/posts/:post_id/likes", post(like_post))
        .route("/api/posts/:post_id", get(get_post).delete(delete_post))
        .route("/api/posts", get(get_posts).post(create_post))
        .route("/api/users/:user_id", get(get_user_posts))
        .layer(axum::middleware::from_fn(validate_jwt));

    Router::new()
        .merge(secure_router)
        .merge(router)
        .nest_service("/static", ServeDir::new("static"))
        .fallback(handle_404)
        .with_state(state)
}

#[derive(Debug, Template)]
#[template(path = "index.askama.html")]
struct IndexTemplate {}

/// `GET /`
async fn get_page_index(_: State<Repository>) -> Result<impl IntoResponse, AppError> {
    info!("Index page was requested.");
    let html = IndexTemplate {};

    Ok(askama_axum::into_response(&html))
}

#[derive(Debug, Template)]
#[template(path = "register.askama.html")]
struct RegisterTemplate {}

/// `GET /register`
async fn get_page_registration(_: State<Repository>) -> Result<impl IntoResponse, AppError> {
    info!("Register page was requested.");
    let html = RegisterTemplate {};

    Ok(askama_axum::into_response(&html))
}

#[derive(Debug, Template)]
#[template(path = "login.askama.html")]
struct LoginTemplate {}

/// `GET /login`
async fn get_page_login(_: State<Repository>) -> Result<impl IntoResponse, AppError> {
    info!("Login page was requested.");
    let html = LoginTemplate {};

    Ok(askama_axum::into_response(&html))
}

#[derive(Debug, Template)]
#[template(path = "posts.askama.html")]
struct PostsTemplate {}

/// `GET /posts`
async fn get_page_posts(_: State<Repository>) -> Result<impl IntoResponse, AppError> {
    info!("Posts page was requested.");
    let html = PostsTemplate {};

    Ok(askama_axum::into_response(&html))
}

#[derive(Debug, Template)]
#[template(path = "post.askama.html")]
struct PostTemplate {}

/// `GET /posts/{post_id}`
async fn get_page_post(
    _: State<Repository>,
    Path(post_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    info!("Post page with id = {post_id} was requested.");
    let html = PostTemplate {};

    Ok(askama_axum::into_response(&html))
}

#[derive(Debug, Template)]
#[template(path = "user.askama.html")]
struct UserTemplate {}

async fn get_page_user(
    _: State<Repository>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    info!("User page with id = {user_id} was requested.");
    let html = UserTemplate {};

    Ok(askama_axum::into_response(&html))
}

/// `POST /api/register`
async fn register_user(
    State(pool): State<Repository>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        "Register request received for username: '{}'",
        payload.username
    );

    if payload.is_empty() {
        warn!("Register request failed: Credentials are empty");
        return Err(AppError::authenthication("Credentials are empty"));
    }

    let RegisterRequest { username, password } = payload;

    info!("Hashing password for username: '{}'", username);

    let password_hash = PasswordHash::from_password(&password)?;

    info!("Registering user with username: '{}'", username);

    pool.register_user(&username, password_hash).await?;

    Ok(Json(
        json!({ "result": "ok", "message": "Успешная регистрация!" }),
    ))
}

/// `POST /api/login`
async fn login_user(
    State(pool): State<Repository>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Login request received for username: {}", payload.username);

    if payload.is_empty() {
        warn!("Login request failed: Credentials are empty");
        return Err(AppError::authenthication("Credentials are empty"));
    }

    let LoginRequest { username, password } = payload;

    info!("Fetching login credentials for username: '{}'", username);

    let Some(user) = pool.get_login_credentials(&username).await? else {
        warn!("User not found in database: '{}'", username);
        return Err(AppError::user_not_found());
    };

    info!("Verifying password for username: '{}'", username);

    if !user.password_hash.verify_password(&password)? {
        warn!("Password verification failed for username: '{}'", username);
        return Err(AppError::authenthication("Wrong password"));
    }

    info!(
        "Password verified successfully for username: '{}'",
        username
    );

    let user_id = user.user_id;

    let token = auth::create_access_token(user_id)?;

    let jwt = json!({
        "token": token,
        "username": username,
        "user_id": user_id,
    });

    info!("Login successful for username: '{}'", username);

    Ok(Json(json!({ "result": "ok", "jwt": jwt })))
}

/// `GET /api/posts`
async fn get_posts(
    State(pool): State<Repository>,
    _: Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    info!("List all posts was requested.");
    let posts = pool.get_posts().await?;

    Ok(Json(json!({ "result": "ok", "posts": posts })))
}

/// `POST /api/posts`
async fn create_post(
    State(pool): State<Repository>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Create post was requested.");

    let Claims { sub: user_id, .. } = claims;
    let CreatePostRequest { title, content } = payload;

    let post_id = pool.create_post(user_id, &title, &content).await?;

    Ok(Json(json!({ "result": "ok", "post_id": post_id })))
}

/// `GET /api/posts/{post_id}`
async fn get_post(
    State(pool): State<Repository>,
    Extension(_): Extension<Claims>,
    Path(post_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    info!("Get post was requested.");

    let Some(post) = pool.get_post(post_id).await? else {
        return Err(AppError::post_not_found());
    };

    Ok(Json(json!({ "result": "ok", "post": post })))
}

/// `POST /api/posts/{post_id}/likes`
async fn like_post(
    State(pool): State<Repository>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    info!("Add/remove like was requested.");

    let Claims { sub: user_id, .. } = claims;
    let like = pool.like_post(user_id, post_id).await?;

    let likes_count = pool.get_like_count(post_id).await?;

    Ok(Json(
        json!({"result": "ok", "like": like, "likes_count": likes_count }),
    ))
}

/// `POST /api/posts/{post_id}`
async fn delete_post(
    State(pool): State<Repository>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    info!("Delete post was requested.");

    let Claims { sub: user_id, .. } = claims;

    match pool.delete_post(post_id, user_id).await? {
        PostDeleteResult::Deleted => Ok(Json(json!({
            "result": "ok",
            "message": "Post deleted successfully."
        }))),
        PostDeleteResult::NotFound => Err(AppError::post_not_found()),
        PostDeleteResult::NotOwned => Err(AppError::forbidden(
            "You do not have permission to delete this post.",
        )),
    }
}

async fn get_user_posts(
    State(pool): State<Repository>,
    _: Extension<Claims>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    info!("User posts list was requested.");

    let Some(username) = pool.get_username_by_user_id(user_id).await? else {
        return Err(AppError::user_not_found());
    };
    let posts = pool.get_user_posts(user_id).await?;

    Ok(Json(
        json!({ "result": "ok", "username": username, "posts": posts }),
    ))
}

async fn handle_404() -> AppError {
    info!("User tried to access non-existing page");

    AppError::page_not_found()
}
