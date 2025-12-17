use actix_web::dev::ServiceRequest;
use actix_web::{Error, error::ErrorUnauthorized};
use uuid::Uuid;

use crate::utils::auth::decode_jwt;

#[allow(dead_code)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

#[allow(dead_code)]
pub fn extract_user_from_request(req: &ServiceRequest, jwt_secret: &str) -> Result<AuthenticatedUser, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ErrorUnauthorized("Missing authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ErrorUnauthorized("Invalid authorization format"))?;

    let claims = decode_jwt(token, jwt_secret)
        .map_err(|_| ErrorUnauthorized("Invalid or expired token"))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ErrorUnauthorized("Invalid user ID in token"))?;

    Ok(AuthenticatedUser { user_id })
}
