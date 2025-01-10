use axum::{extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{encode, DecodingKey, EncodingKey};
use std::sync::LazyLock;
use ulid::Ulid;

use super::{AppError, Claims};

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
    let jwt_token = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let jwt_token = match jwt_token {
        Some(jwt_token) => {
            dbg!(jwt_token);
            jwt_token.replace("Bearer ", "")
        }
        None => {
            return Err(AppError::Authenthication(
                "Authorization token is missing".to_owned(),
            ));
        }
    };

    let token_payload = jsonwebtoken::decode::<Claims>(
        &jwt_token,
        &KEYS.decoding,
        &jsonwebtoken::Validation::default(),
    );

    match token_payload {
        Ok(token_payload) => {
            req.extensions_mut().insert(token_payload.claims);
            Ok(next.run(req).await)
        }
        Err(err) => Err(AppError::JwtToken(err)),
    }
}

pub(crate) fn create_access_token(user_id: Ulid) -> Result<String, AppError> {
    let expires = usize::try_from(
        chrono::Utc::now()
            .checked_add_days(chrono::Days::new(1))
            .unwrap()
            .timestamp(),
    )
    .unwrap();

    let claims = Claims {
        sub: user_id,
        exp: expires,
    };

    let token = encode(&jsonwebtoken::Header::default(), &claims, &KEYS.encoding)?;

    Ok(token)
}
