pub mod config;
mod jwt;
mod middleware;

use actix_web::{web, HttpResponse};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use validator::Validate;

use crate::{db::Database, error::AppError, models::*};

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn register(db: web::Data<Database>, user: web::Json<CreateUserRequest>) -> Result<HttpResponse, AppError> {
    if let Err(e) = user.validate() {
        return Err(AppError::ValidationError(e.to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(user.password.as_bytes(), &salt)
        .map_err(|e| AppError::AuthError(e.to_string()))?
        .to_string();

    let user = db.create_user(&user, password_hash).await?;
    let token = jwt::create_token(user.id)?;

    Ok(HttpResponse::Ok().json(AuthResponse { token, user }))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(db: web::Data<Database>, creds: web::Json<LoginRequest>) -> Result<HttpResponse, AppError> {
    let user = db.get_user_by_email(&creds.email).await?;

    let argon2 = Argon2::default();
    let parsed_hash = argon2::PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::AuthError(e.to_string()))?;

    if argon2
        .verify_password(creds.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AppError::AuthError("Invalid credentials".to_string()));
    }

    let token = jwt::create_token(user.id)?;

    Ok(HttpResponse::Ok().json(AuthResponse { token, user }))
}