use super::{AppError, Claims};
use axum::{extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{encode, DecodingKey, EncodingKey};
use log::{error, info, warn};
use std::sync::LazyLock;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub(crate) async fn validate_jwt(mut req: Request, next: Next) -> Result<Response, AppError> {
    info!("Starting JWT validation for request.");

    let jwt_token = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let jwt_token = if let Some(jwt_token) = jwt_token {
        info!("JWT token found in the Authorization header.");
        jwt_token.replace("Bearer ", "")
    } else {
        warn!("No JWT token found in the Authorization header.");
        return Err(AppError::authenthication(
            "You cannot access or interact with the content of this page. Please log in to continue.",
        ));
    };

    info!("Decoding and validating JWT token.");
    let token_payload = jsonwebtoken::decode::<Claims>(
        &jwt_token,
        &KEYS.decoding,
        &jsonwebtoken::Validation::default(),
    );

    match token_payload {
        Ok(token_payload) => {
            info!("JWT token successfully validated.");
            req.extensions_mut().insert(token_payload.claims);
            Ok(next.run(req).await)
        }
        Err(err) => {
            error!("JWT token validation failed: {:?}", err);
            Err(AppError::jwt_token(err))
        }
    }
}

pub(crate) fn create_access_token(user_id: i32) -> Result<String, AppError> {
    info!("Creating access token for user_id: {}", user_id);

    let expires = if let Some(expiry) = chrono::Utc::now().checked_add_days(chrono::Days::new(1)) {
        info!(
            "Token expiry calculated successfully for user_id: '{}'",
            user_id
        );
        usize::try_from(expiry.timestamp()).unwrap()
    } else {
        error!(
            "Failed to calculate token expiry for user_id: '{}'",
            user_id
        );
        return Err(AppError::other(anyhow::anyhow!(
            "Failed to calculate token expiry"
        )));
    };

    let claims = Claims {
        sub: user_id,
        exp: expires,
    };

    info!("JWT claims created for user_id: '{}'", user_id);

    let token = match encode(&jsonwebtoken::Header::default(), &claims, &KEYS.encoding) {
        Ok(token) => {
            info!(
                "Access token created successfully for user_id: '{}'",
                user_id
            );
            token
        }
        Err(err) => {
            error!(
                "Failed to create access token for user_id: '{}': {:?}",
                user_id, err
            );
            return Err(AppError::jwt_token(err));
        }
    };

    Ok(token)
}
