use actix_web::{
  Scope,
  web::{self},
};

use crate::{
  features::users::{
    user_entity::UserRole,
    user_handler::{get_user_by_id, get_users},
  },
  middleware::auth::RequireAuth,
};

pub fn user_routes() -> Scope {
  web::scope("/users")
    .route(
      "/all",
      web::post()
        .to(get_users)
        .wrap(RequireAuth::allow_roles(vec![UserRole::Admin])),
    )
    .route(
      "/by_id",
      web::post()
        .to(get_user_by_id)
        .wrap(RequireAuth::allow_roles(vec![
          UserRole::User,
          UserRole::Moderator,
          UserRole::Admin,
        ])),
    )
}
