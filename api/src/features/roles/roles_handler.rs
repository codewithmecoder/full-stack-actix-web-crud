use actix_web::{HttpResponse, Responder, web};

use crate::{
  app_state::AppState,
  dto::base_res_dto::Status,
  error::StatusMessage,
  features::roles::{
    roles_dto::{CreateRoleReqDto, UpdateRoleReqDto},
    roles_entity::RoleEntity,
    roles_repo::RoleRepo,
  },
};

pub async fn create_role(
  role: web::Json<CreateRoleReqDto>,
  data: web::Data<AppState>,
) -> impl Responder {
  let mut repo = RoleRepo::new(&data);

  match repo.get_by_name(&role.name).await {
    Ok(role_existed) => {
      if let Some(_) = role_existed {
        return Status::bad_request(StatusMessage::Existed(format!(
          "Role with name '{}'",
          role.name
        )))
        .into_http_response();
      }

      let entity = RoleEntity::from(role.into_inner());
      match repo.create_role(&entity).await {
        Ok(_) => HttpResponse::Ok().json(Status::success()),
        Err(e) => Status::bad_request(format!("Failed to create role: {}", e)).into_http_response(),
      }
    }
    Err(e) => Status::bad_request(format!("Failed to create role: {}", e)).into_http_response(),
  }
}

pub async fn update_role(
  role: web::Json<UpdateRoleReqDto>,
  data: web::Data<AppState>,
) -> impl Responder {
  let mut repo = RoleRepo::new(&data);

  match repo.get_by_id(role.id).await {
    Ok(role_existed) => {
      if let Some(_) = role_existed {
        if let Ok(Some(_)) = repo.get_by_name(&role.name).await {
          return Status::bad_request(
            StatusMessage::Existed(format!("Role name '{}'", role.name)).to_str(),
          )
          .into_http_response();
        }

        let entity = RoleEntity::from(role.into_inner());
        return match repo.update_role(&entity).await {
          Ok(_) => HttpResponse::Ok().json(Status::success()),
          Err(e) => {
            Status::bad_request(format!("Failed to update role: {}", e)).into_http_response()
          }
        };
      }
      Status::not_found(StatusMessage::NotFound(format!("Role with id '{}'", role.id)).to_str())
        .into_http_response()
    }
    Err(e) => Status::bad_request(format!("failed to update role: {}", e)).into_http_response(),
  }
}
