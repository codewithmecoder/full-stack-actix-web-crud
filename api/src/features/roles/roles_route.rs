use actix_web::{Scope, web};

use crate::features::roles::roles_handler::{create_role, update_role};
use crate::{features::users::user_entity::UserRole, middleware::auth::RequireAuth};
pub fn role_routes() -> Scope {
  web::scope("/role")
    // .route(
    //   "/all",
    //   web::post()
    //     .to(get_users)
    //     .wrap(RequireAuth::allow_roles(vec![UserRole::Admin])),
    // )
    // .route(
    //   "/by_id",
    //   web::post()
    //     .to(get_user_by_id)
    //     .wrap(RequireAuth::allow_roles(vec![
    //       UserRole::User,
    //       UserRole::Moderator,
    //       UserRole::Admin,
    //     ])),
    // )
    .route(
      "/create",
      web::post()
        .to(create_role)
        .wrap(RequireAuth::allow_roles(vec![UserRole::Admin])),
    )
    .route(
      "/update",
      web::post()
        .to(update_role)
        .wrap(RequireAuth::allow_roles(vec![UserRole::Admin])),
    )
}
