use actix_web::{HttpResponse, Responder, web};

use crate::{
  app_state::AppState,
  commons::status_code_const::StatusCodeConst,
  dto::base_res_dto::{BaseResDto, Status},
  error::StatusMessage,
  features::users::{
    user_dto::{GetUserByIdReqDto, UserDto},
    user_repo::UserRepo,
  },
};

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
        HttpResponse::BadRequest().json(Status::bad_request(StatusMessage::NotFound("User".into())))
      }
    }
    Err(e) => {
      HttpResponse::BadRequest().json(Status::bad_request(format!("Failed to get users: {}", e)))
    }
  }
}
