use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

use super::{middleware, register, login};

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