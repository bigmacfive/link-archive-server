use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod utils;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::register,
        auth::login,
        handlers::links::create_link,
        handlers::links::get_links,
        handlers::links::get_link,
        handlers::links::update_link,
        handlers::links::delete_link
    ),
    components(
        schemas(models::User, models::CreateUserRequest, models::LoginRequest,
               models::Link, models::CreateLinkRequest, models::UpdateLinkRequest,
               models::Tag, models::AuthResponse, models::LinkResponse)
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "links", description = "Link management endpoints")
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let server_port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| String::from("8080"))
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .configure(auth::config::configure)
            .configure(handlers::links::configure)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
    })
    .bind(("127.0.0.1", server_port))?
    .run()
    .await
}