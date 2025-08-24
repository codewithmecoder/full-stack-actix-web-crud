mod app_settings;
mod app_state;
mod commons;
mod dto;
mod features;
mod repos;

use actix_web::{App, HttpServer, web};

use crate::{
  app_state::AppState,
  features::{auth::auth_handler, users::user_handler},
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

  let server = HttpServer::new(move || {
    App::new()
      .app_data(state.clone())
      // Public routes here
      .service(
        web::scope("/api/v1")
          .service(web::scope("/auth").service(auth_handler::register))
          .service(
            web::scope("/users")
              .service(user_handler::get_users)
              .service(user_handler::get_user_by_id),
          ),
      )
  })
  .bind((host.clone(), port))?;
  // Log the running address
  println!("Server is running at http://{}:{}", host, port);

  // Run the server (blocking)
  server.run().await
}
