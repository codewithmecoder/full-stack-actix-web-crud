use actix_web::{HttpResponse, Responder, web};

use crate::{
  app_state::AppState,
  commons::status_code_const::StatusCodeConst,
  dto::base_res_dto::{BaseResDto, Status},
  error::StatusMessage,
  features::users::{
    user_dto::{GetUserByIdReqDto, UpdateUserReqDto, UserDto},
    user_entity::UserRole,
    user_repo::UserRepo,
  },
};

#[utoipa::path(
    post,
    path = "/api/v1/user/all",
    tag = "Users",
    request_body(
        content = (),
        description = "",
        example = json!({})),
    responses( 
        (
            status=200, 
            description= "Get users successfully", 
            body= BaseResDto<Vec<UserDto>>
        ),
        (
            status=400, 
            description= "Validation Errors", 
            body= Status
        ),
        (
            status=500, 
            description= "Internal Server Error", 
            body= Status 
        ),
    )
)]
pub async fn get_users(data: web::Data<AppState>) -> impl Responder {
  let mut repo = UserRepo::new(&data);

  match repo.get_users().await {
    Ok(users) => HttpResponse::Ok().json(BaseResDto::<Vec<UserDto>> {
      data: Some(users.into_iter().map(|u| UserDto::from(u)).collect()),
      status: Status {
        message: "Users retrieved successfully".to_string(),
        code: StatusCodeConst::SUCCESS.to_string(),
        status: 200,
      },
    }),
    Err(e) => {
      HttpResponse::BadRequest().json(Status::bad_request(format!("Failed to get users: {}", e)))
    }
  }
}

#[utoipa::path(
    post,
    path = "/api/v1/user/by_id",
    tag = "Users",
    request_body(
        content = GetUserByIdReqDto,
        description = "",
        example = json!({
          "id": 1
        })),
    responses( 
        (
            status=200, 
            description= "Get users successfully", 
            body= BaseResDto<UserDto>
        ),
        (
            status=400, 
            description= "Validation Errors", 
            body= Status
        ),
        (
            status=500, 
            description= "Internal Server Error", 
            body= Status 
        ),
    )
)]
pub async fn get_user_by_id(
  id: web::Json<GetUserByIdReqDto>,
  data: web::Data<AppState>,
) -> impl Responder {
  let mut repo = UserRepo::new(&data);

  match repo.get_by_id(id.id).await {
    Ok(user) => {
      if let Some(u) = user {
        HttpResponse::Ok().json(Status::success_with_data(UserDto::from(u)))
      } else {
        HttpResponse::BadRequest().json(Status::not_found(StatusMessage::NotFound("User".into())))
      }
    }
    Err(e) => {
      HttpResponse::BadRequest().json(Status::bad_request(format!("Failed to get users: {}", e)))
    }
  }
}

#[utoipa::path(
    post,
    path = "/api/v1/user/update",
    tag = "Users",
    request_body(
        content = UpdateUserReqDto,
        description = "",
        example = json!(
          {
            "user_name": "nith",
            "name": "nith update",
            "email": "nithupdate@gmail.com",
            "role": "admin"
          })),
    responses( 
        (
            status=200, 
            description= "Update user successfully", 
            body= Status
        ),
        (
            status=400, 
            description= "Validation Errors", 
            body= Status
        ),
        (
            status=500, 
            description= "Internal Server Error", 
            body= Status 
        ),
    )
)]
pub async fn update_user(
  user_update: web::Json<UpdateUserReqDto>,
  data: web::Data<AppState>,
) -> impl Responder {
  let mut repo = UserRepo::new(&data);

  match repo.get_by_username(&user_update.user_name).await {
    Ok(user) => {
      if let Some(mut u) = user {
        if let Some(new_email) = &user_update.email {
          u.email = new_email.clone();
        }
        if let Some(new_name) = &user_update.name {
          u.name = new_name.clone();
        }
        if let Some(new_role) = &user_update.role {
          let parsed_role = UserRole::from_str(new_role);
          u.role = parsed_role;
        }

        match repo.update_user(&UserDto::from(u.clone())).await {
          Ok(_) => HttpResponse::Ok().json(Status::success()),
          Err(e) => HttpResponse::BadRequest()
            .json(Status::bad_request(format!("Failed to update user: {}", e))),
        }
      } else {
        HttpResponse::BadRequest().json(Status::not_found(StatusMessage::NotFound("User".into())))
      }
    }
    Err(e) => {
      HttpResponse::BadRequest().json(Status::bad_request(format!("Failed to update user: {}", e)))
    }
  }
}
