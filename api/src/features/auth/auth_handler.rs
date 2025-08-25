use actix_web::{
  HttpResponse, Responder,
  cookie::{
    Cookie,
    time::{Duration, OffsetDateTime},
  },
  post, web,
};

use crate::{
  app_state::AppState,
  commons::status_code_const::StatusCodeConst,
  dto::base_res_dto::{BaseResDto, Status},
  features::{
    auth::auth_dto::{LoginReqDto, LoginResDto},
    users::{
      user_dto::{UserDto, UserRegisterReqDto},
      user_repo::UserRepo,
    },
  },
  utils::{jwt_util::JwtUtil, password_hashing::PasswordHashing},
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

#[post("/login")]
pub async fn login(user: web::Json<LoginReqDto>, data: web::Data<AppState>) -> impl Responder {
  let mut repo = UserRepo::new(&data);
  if user.user_name.is_empty() || user.password.is_empty() {
    return HttpResponse::Unauthorized().json(BaseResDto::<UserDto> {
      data: None,
      status: Status {
        message: "Invalid credentials".to_string(),
        code: StatusCodeConst::UNAUTHORIZED.to_string(),
      },
    });
  }

  if let Ok(Some(db_user)) = repo.get_by_username(&user.user_name).await {
    if PasswordHashing::verify_password(&user.password, &db_user.password) {
      let jwt_util = JwtUtil::new(&data.config.jwt);
      if let Ok(token) = jwt_util.create_token(&UserDto::from(db_user)) {
        let now = OffsetDateTime::now_utc();
        let expiration = now + Duration::minutes(data.config.jwt.expiration_minutes as i64);
        let cookie = Cookie::build("auth", &token)
          .path("/")
          .http_only(true)
          .expires(expiration)
          .finish();
        return HttpResponse::Ok()
          .cookie(cookie)
          .json(BaseResDto::<LoginResDto> {
            data: Some(LoginResDto { token }),
            status: Status {
              message: "Login Successfully".to_string(),
              code: StatusCodeConst::UNAUTHORIZED.to_string(),
            },
          });
      }
    }
  }

  HttpResponse::Ok().json(BaseResDto::<LoginResDto> {
    data: None,
    status: Status {
      message: "Invalid credentials".to_string(),
      code: StatusCodeConst::UNAUTHORIZED.to_string(),
    },
  })
}
