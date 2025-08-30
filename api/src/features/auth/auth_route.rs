use actix_web::{
  Scope,
  web::{self},
};

use crate::{
  features::{
    auth::auth_handler::{login, logout, register},
    users::user_entity::UserRole,
  },
  middleware::auth::RequireAuth,
};

pub fn auth_routes() -> Scope {
  web::scope("/auth")
    .route("/register", web::post().to(register))
    .route("/login", web::post().to(login))
    .route(
      "/logout",
      web::post().to(logout).wrap(RequireAuth::allow_roles(vec![
        UserRole::User,
        UserRole::Moderator,
        UserRole::Admin,
      ])),
    )
}
