use actix_web::{web, HttpResponse};
use uuid::Uuid;
use validator::Validate;

use crate::{db::Database, error::AppError, models::*};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/links")
            .wrap(crate::auth::config::auth_middleware())
            .route("", web::post().to(create_link))
            .route("", web::get().to(get_links))
            .route("/{id}", web::get().to(get_link))
            .route("/{id}", web::put().to(update_link))
            .route("/{id}", web::delete().to(delete_link))
    );
}

#[utoipa::path(
    post,
    path = "/api/links",
    request_body = CreateLinkRequest,
    responses(
        (status = 201, description = "Link created successfully", body = LinkResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    )
)]
async fn create_link(
    db: web::Data<Database>,
    user_id: web::ReqData<Uuid>,
    link: web::Json<CreateLinkRequest>,
) -> Result<HttpResponse, AppError> {
    if let Err(e) = link.validate() {
        return Err(AppError::ValidationError(e.to_string()));
    }

    let link = db.create_link(*user_id, &link).await?;
    Ok(HttpResponse::Created().json(link))
}

#[utoipa::path(
    get,
    path = "/api/links",
    responses(
        (status = 200, description = "List of user's links", body = Vec<LinkResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    )
)]
async fn get_links(
    db: web::Data<Database>,
    user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    let links = db.get_user_links(*user_id).await?;
    Ok(HttpResponse::Ok().json(links))
}

#[utoipa::path(
    get,
    path = "/api/links/{id}",
    responses(
        (status = 200, description = "Link details", body = LinkResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Link not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Link ID")
    ),
    security(
        ("bearer" = [])
    )
)]
async fn get_link(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let link = db.get_link(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(link))
}

#[utoipa::path(
    put,
    path = "/api/links/{id}",
    request_body = UpdateLinkRequest,
    responses(
        (status = 200, description = "Link updated successfully", body = LinkResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Link not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Link ID")
    ),
    security(
        ("bearer" = [])
    )
)]
async fn update_link(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    update: web::Json<UpdateLinkRequest>,
) -> Result<HttpResponse, AppError> {
    let link = db.update_link(path.into_inner(), &update).await?;
    Ok(HttpResponse::Ok().json(link))
}

#[utoipa::path(
    delete,
    path = "/api/links/{id}",
    responses(
        (status = 204, description = "Link deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Link not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, Path, description = "Link ID")
    ),
    security(
        ("bearer" = [])
    )
)]
async fn delete_link(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    db.delete_link(path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}