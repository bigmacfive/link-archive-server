use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_httpauth::extractors::bearer::Bearer;

use super::{middleware::{self, auth_validator}, register, login};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    );
}

pub fn auth_middleware() -> HttpAuthentication<Bearer, auth_validator> {
    HttpAuthentication::bearer(middleware::auth_validator)
}