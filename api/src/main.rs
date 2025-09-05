mod app_settings;
mod app_state;
mod commons;
mod dto;
mod error;
mod features;
mod middleware;
mod repos;
mod swaggers;
mod utils;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http::header, middleware::Logger, web};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
  app_state::AppState,
  features::{
    auth::auth_route::auth_routes, health_check::health_checker_handler,
    roles::roles_route::role_routes, users::user_route::user_routes,
  },
  swaggers::ApiDoc,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  unsafe {
    openssl_probe::init_openssl_env_vars();
  }
  // Load AppState from JSON file
  let state = match AppState::load_setting("appsettings.json").await {
    Ok(state) => web::Data::new(state),
    Err(e) => {
      eprintln!("Failed to initialize app state: {}", e);
      std::process::exit(1);
    }
  };

  unsafe {
    if std::env::var_os("RUST_LOG").is_none() {
      std::env::set_var("RUST_LOG", format!("actix_web={}", state.config.rust_log));
    }
  }

  let host = state.config.server.host.clone();
  let port = state.config.server.port;
  let open_api = ApiDoc::openapi();
  let server = HttpServer::new(move || {
    let cors = Cors::default()
      .allowed_origin("http://localhost:3000")
      .allowed_origin("http://localhost:8000")
      .allowed_methods(vec!["GET", "POST"])
      .allowed_headers(vec![
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        header::ACCEPT,
      ])
      .supports_credentials();
    App::new()
      .app_data(state.clone())
      .wrap(cors)
      .wrap(Logger::default())
      // Public routes here
      .service(
        web::scope("/api/v1")
          .service(health_checker_handler)
          .service(auth_routes())
          .service(user_routes())
          .service(role_routes()),
      )
      .service(Redoc::with_url("/redoc", open_api.clone()))
      .service(RapiDoc::new("/api-docs/openapi.json").path("/redoc"))
      .service(SwaggerUi::new("/{_:.*}").url("/api-docs/openapi.json", open_api.clone()))
  })
  .bind((host.clone(), port))?;
  // Log the running address
  println!("Server is running at http://{}:{}", host, port);

  // Run the server (blocking)
  server.run().await
}
