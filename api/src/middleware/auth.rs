use std::{ops, rc::Rc};

use actix_web::{
  FromRequest, HttpMessage, body,
  dev::{Service, ServiceRequest, ServiceResponse, Transform},
  error::{ErrorForbidden, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
  http, web,
};
use futures::{
  FutureExt,
  future::{LocalBoxFuture, Ready, ready},
};

use crate::{
  app_state::AppState,
  dto::base_res_dto::Status,
  error::StatusMessage,
  features::users::{user_dto::UserDto, user_entity::UserRole, user_repo::UserRepo},
  utils::jwt_util::JwtUtil,
};

pub struct Authenticated(UserDto);

impl FromRequest for Authenticated {
  type Error = actix_web::Error;

  type Future = Ready<Result<Self, Self::Error>>;

  fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
    let value = req.extensions().get::<UserDto>().cloned();
    let result = match value {
      Some(user) => Ok(Authenticated(user)),
      None => Err(ErrorInternalServerError(Status::server_error(
        "Authentication error",
      ))),
    };
    ready(result)
  }
}

impl ops::Deref for Authenticated {
  type Target = UserDto;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub struct RequireAuth {
  pub allow_roles: Rc<Vec<UserRole>>,
}

impl RequireAuth {
  pub fn allow_roles(allow_roles: Vec<UserRole>) -> Self {
    Self {
      allow_roles: Rc::new(allow_roles),
    }
  }
}

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
  S: Service<ServiceRequest, Response = ServiceResponse<body::BoxBody>, Error = actix_web::Error>
    + 'static,
{
  type Response = ServiceResponse<body::BoxBody>;

  type Error = actix_web::Error;

  type Transform = AuthMiddleware<S>;

  type InitError = ();

  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(AuthMiddleware {
      service: Rc::new(service),
      allow_roles: self.allow_roles.clone(),
    }))
  }
}

pub struct AuthMiddleware<S> {
  service: Rc<S>,
  allow_roles: Rc<Vec<UserRole>>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<body::BoxBody>, Error = actix_web::Error>
    + 'static,
{
  type Response = ServiceResponse<body::BoxBody>;

  type Error = actix_web::Error;

  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(
    &self,
    ctx: &mut core::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self.service.poll_ready(ctx)
  }

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let app_state = req.app_data::<web::Data<AppState>>().unwrap();

    let token = req
      .cookie(&app_state.config.cookie.name)
      .map(|c| c.value().to_string())
      .or_else(|| {
        req
          .headers()
          .get(http::header::AUTHORIZATION)
          .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
      });
    if token.is_none() {
      return Box::pin(ready(Err(ErrorUnauthorized(Status::token_missing()))));
    }

    let jwt_util = JwtUtil::new(&app_state.config.jwt);
    let user_claims = match jwt_util.decode_token(&token.unwrap()) {
      Ok(claims) => claims,
      Err(e) => {
        return Box::pin(ready(Err(ErrorUnauthorized(Status::unauthorized(
          format!("{}, {}", StatusMessage::DecodeTokenErr.to_str(), e),
        )))));
      }
    };

    let app_state_cloned = app_state.clone();
    let allow_roles = self.allow_roles.clone();
    let srv = Rc::clone(&self.service);

    async move {
      let mut user_repo = UserRepo::new(&app_state_cloned);
      let result = user_repo
        .get_by_id(user_claims.sub)
        .await
        .map_err(|e| ErrorInternalServerError(Status::server_error(e.to_string())))?;

      let user = result.ok_or(ErrorNotFound(Status::not_found("User")))?;

      if allow_roles.contains(&user.role) {
        req.extensions_mut().insert::<UserDto>(UserDto::from(user));
        let res = srv.call(req).await?;
        Ok(res)
      } else {
        Err(ErrorForbidden(Status::forbidden()))
      }
    }
    .boxed_local()
  }
}
