use actix_web::web::{self, ServiceConfig};

use crate::features::auth::auth_handler;

pub fn auth_routes(cfg: &mut ServiceConfig) {
  cfg.service(
    web::scope("/auth")
      .service(auth_handler::register)
      .service(auth_handler::login),
  );
}
