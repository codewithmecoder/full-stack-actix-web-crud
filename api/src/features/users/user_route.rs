use actix_web::web::{self, ServiceConfig};

use crate::features::users::user_handler;

pub fn user_routes(cfg: &mut ServiceConfig) {
  cfg.service(
    web::scope("/users")
      .service(user_handler::get_users)
      .service(user_handler::get_user_by_id),
  );
}
