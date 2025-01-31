use actix_web::{dev::ServiceRequest, Error, error::ErrorUnauthorized, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use super::jwt;

pub async fn auth_validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    match jwt::verify_token(credentials.token()) {
        Ok(user_id) => {
            req.extensions_mut().insert(user_id);
            Ok(req)
        }
        Err(_) => Err(ErrorUnauthorized("Invalid token"))
    }
}