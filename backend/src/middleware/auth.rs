use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, error::ErrorUnauthorized};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use uuid::Uuid;

use crate::utils::auth::decode_jwt;
use crate::utils::config::Config;

// Re-export Claims for use in handlers
pub use crate::utils::auth::Claims;

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

// JWT Middleware implementation
pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            log::debug!("🔐 [JWT Middleware] Checking authentication for: {} {}", req.method(), req.path());

            // Extract JWT secret from app data
            let config = req.app_data::<actix_web::web::Data<Config>>()
                .ok_or_else(|| {
                    log::error!("❌ [JWT Middleware] Configuration not found");
                    ErrorUnauthorized("Configuration not found")
                })?;

            let jwt_secret = &config.jwt_secret;

            // Extract and validate token
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| {
                    log::warn!("⚠️  [JWT Middleware] Missing authorization header");
                    ErrorUnauthorized("Missing authorization header")
                })?;

            log::debug!("🔑 [JWT Middleware] Authorization header present");

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or_else(|| {
                    log::error!("❌ [JWT Middleware] Invalid authorization format (missing 'Bearer ')");
                    ErrorUnauthorized("Invalid authorization format")
                })?;

            log::debug!("🔍 [JWT Middleware] Decoding JWT token");

            let claims = decode_jwt(token, jwt_secret)
                .map_err(|e| {
                    log::error!("❌ [JWT Middleware] JWT decode error: {}", e);
                    ErrorUnauthorized("Invalid or expired token")
                })?;

            log::debug!("✅ [JWT Middleware] JWT validated for user: {}", claims.sub);

            // Insert claims into request extensions for handlers to access
            req.extensions_mut().insert(claims);

            // Call the next service
            service.call(req).await
        })
    }
}
