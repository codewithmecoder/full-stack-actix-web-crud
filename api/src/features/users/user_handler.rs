use actix_web::{HttpResponse, Responder, post, web};

use crate::{
  app_state::AppState,
  commons::status_code_const::StatusCodeConst,
  features::{
    dto::base_res_dto::{BaseResDto, Status},
    users::{user_repo::UserRepo, user_req_dto::GetUserByIdReqDto, user_res_dto::UserDto},
  },
};

#[post("/")]
pub async fn get_users(data: web::Data<AppState>) -> impl Responder {
  let mut repo = UserRepo::new(&data);

  match repo.get_users().await {
    Ok(users) => HttpResponse::Ok().json(web::Json(BaseResDto::<Vec<UserDto>> {
      data: Some(users.into_iter().map(|u| UserDto::from(u)).collect()),
      status: Status {
        message: "Users retrieved successfully".to_string(),
        code: StatusCodeConst::SUCCESS.to_string(),
      },
    })),
    Err(e) => HttpResponse::BadRequest().json(web::Json(BaseResDto::<Vec<UserDto>> {
      data: None,
      status: Status {
        message: format!("Failed to get users: {}", e),
        code: StatusCodeConst::ERROR.to_string(),
      },
    })),
  }
}

#[post("/by_id")]
pub async fn get_user_by_id(
  id: web::Json<GetUserByIdReqDto>,
  data: web::Data<AppState>,
) -> impl Responder {
  let mut repo = UserRepo::new(&data);

  match repo.get_by_id(id.id).await {
    Ok(user) => {
      if let Some(u) = user {
        HttpResponse::Ok().json(web::Json(BaseResDto::<UserDto> {
          data: Some(UserDto::from(u)),
          status: Status {
            message: "User retrieved successfully".to_string(),
            code: StatusCodeConst::SUCCESS.to_string(),
          },
        }))
      } else {
        HttpResponse::BadRequest().json(web::Json(BaseResDto::<UserDto> {
          data: None,
          status: Status {
            message: "User not found".to_string(),
            code: StatusCodeConst::NOT_FOUND.to_string(),
          },
        }))
      }
    }
    Err(e) => HttpResponse::BadRequest().json(web::Json(BaseResDto::<Vec<UserDto>> {
      data: None,
      status: Status {
        message: format!("Failed to get users: {}", e),
        code: StatusCodeConst::ERROR.to_string(),
      },
    })),
  }
}
