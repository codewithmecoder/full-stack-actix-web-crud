use actix_web::{HttpResponse, Responder, post, web};

use crate::{
  app_state::AppState,
  commons::status_code_const::StatusCodeConst,
  dto::base_res_dto::{BaseResDto, Status},
  features::users::{user_repo::UserRepo, user_req_dto::UserRegisterReqDto, user_res_dto::UserDto},
};

#[post("/register")]
pub async fn register(
  user: web::Json<UserRegisterReqDto>,
  data: web::Data<AppState>,
) -> impl Responder {
  let mut repo = UserRepo::new(&data);
  if let Err(e) = repo.create(&user).await {
    return HttpResponse::BadRequest().json(web::Json(BaseResDto::<UserDto> {
      data: None,
      status: Status {
        message: format!("{}", e),
        code: StatusCodeConst::ERROR.to_string(),
      },
    }));
  }
  HttpResponse::Ok().json(BaseResDto::<UserDto> {
    data: None,
    status: Status {
      message: "User registered successfully".to_string(),
      code: StatusCodeConst::SUCCESS.to_string(),
    },
  })
}
