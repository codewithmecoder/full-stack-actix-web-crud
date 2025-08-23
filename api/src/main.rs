mod app_settings;
mod app_state;
mod features;
mod repos;

use actix_web::{App, HttpServer, Responder, web};

use crate::app_state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  unsafe {
    openssl_probe::init_openssl_env_vars();
  }
  // Load AppState from JSON file
  let state = match AppState::from_file("appsettings.json").await {
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
      .route("/", web::get().to(index))
  })
  .bind((host.clone(), port))?;
  // Log the running address
  println!("Server is running at http://{}:{}", host, port);

  // Run the server (blocking)
  server.run().await
}
async fn index(data: web::Data<AppState>) -> impl Responder {
  format!(
    "Server running on {}:{} with DB pool size {}",
    data.config.server.host, data.config.server.port, data.config.database.pool_size
  )
}
