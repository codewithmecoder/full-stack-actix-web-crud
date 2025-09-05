use actix_web::{
  HttpResponse, Responder,
  cookie::{
    self, Cookie,
    time::{Duration, OffsetDateTime},
  },
  web,
};

use crate::{
  app_state::AppState,
  dto::base_res_dto::Status,
  error::StatusMessage,
  features::{
    auth::auth_dto::{LoginReqDto, LoginResDto},
    users::{
      user_dto::{UserDto, UserRegisterReqDto},
      user_repo::UserRepo,
    },
  },
  utils::{jwt_util::JwtUtil, password_hashing::PasswordHashing},
};

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Authentication",
    request_body(
        content = UserRegisterReqDto,
        description = "Credentials to create account",
        example = json!(
            {
                "name": "admin",
                "user_name": "admin",
                "password": "admin",
                "email": "admin@gmail.com",
                "role": "admin"
            })),
    responses( 
        (
            status=200, 
            description= "Account created successfully", 
            body= Status 
        ),
        (
            status=400, 
            description= "Validation Errors", 
            body= Status
        ),
        (
            status=409, 
            description= "User with username already exists", 
            body= Status
        ),
        (
            status=500, 
            description= "Internal Server Error", 
            body= Status 
        ),
    )
)]
pub async fn register(
  user: web::Json<UserRegisterReqDto>,
  data: web::Data<AppState>,
) -> impl Responder {
  if user.password.is_empty()
    || user.user_name.is_empty()
    || user.name.is_empty()
    || user.email.is_empty()
  {
    return HttpResponse::BadRequest().json(Status::uqique_constraint_voilation(
      StatusMessage::WrongParams.to_str(),
    ));
  }

  if user.user_name.len() > 150 {
    return HttpResponse::BadRequest().json(Status::uqique_constraint_voilation(
      StatusMessage::UserNameExeedMaxLength(150).to_str(),
    ));
  }

  let mut repo = UserRepo::new(&data);

  if let Ok(Some(_)) = repo.get_by_username(&user.user_name).await {
    return HttpResponse::Conflict().json(Status::uqique_constraint_voilation(
      StatusMessage::UserNameExisted.to_str(),
    ));
  }

  if let Err(e) = repo.create(&user).await {
    return HttpResponse::BadRequest().json(Status::bad_request(format!("{}", e)));
  }
  HttpResponse::Ok().json(Status::success())
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Authentication",
    request_body(
        content = LoginReqDto,
        description = "",
        example = json!(
            {
                "user_name": "nith",
                "password": "nith"
            })),
    responses( 
        (
            status=200, 
            description= "Login successfully", 
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
pub async fn login(
  user: web::Json<LoginReqDto>,
  data: web::Data<AppState>,
) -> impl Responder {
  let mut repo = UserRepo::new(&data);
  if user.user_name.is_empty() || user.password.is_empty() {
    return HttpResponse::Unauthorized().json(Status::unauthorized(StatusMessage::Unauthorized));
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
          .json(Status::success_with_data(LoginResDto { token }));
      }
    }
  }

  HttpResponse::Ok().json(Status::unauthorized(StatusMessage::Unauthorized))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logut",
    tag = "Authentication",
    request_body(),
    responses( 
        (
            status=200, 
            description= "Logout successfully", 
            body= Status 
        )
    )
)]
pub async fn logout() -> impl Responder {
  let cookie = Cookie::build("token", "")
    .path("/")
    .max_age(cookie::time::Duration::new(-1, 0))
    .http_only(true)
    .finish();

  HttpResponse::Ok().cookie(cookie).json(Status::success())
}
