use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: i64,    // expiration timestamp
    pub iat: i64,    // issued at timestamp
}

impl Claims {
    pub fn new(user_id: Uuid, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(expiration_hours)).timestamp();

        Self {
            sub: user_id.to_string(),
            exp,
            iat: now.timestamp(),
        }
    }
}

pub fn create_jwt(user_id: Uuid, secret: &str, expiration_hours: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id, expiration_hours);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

#[allow(dead_code)]
pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}
