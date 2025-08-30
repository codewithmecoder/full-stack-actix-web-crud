use std::rc::Rc;

use actix_web::{
  Error, HttpMessage, HttpResponse, body,
  dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use anyhow::Result;
use futures::future::{LocalBoxFuture, Ready, ok};

use crate::{app_settings::JwtSetting, utils::jwt_util::JwtUtil};
pub struct JwtAuth {
  pub jwt_config: JwtSetting,
}

impl<S> Transform<S, ServiceRequest> for JwtAuth
where
  S: Service<ServiceRequest, Response = ServiceResponse<body::BoxBody>, Error = Error> + 'static,
{
  type Response = ServiceResponse<body::BoxBody>;

  type Error = Error;

  type Transform = JwtAuthMiddleware<S>;

  type InitError = ();

  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(JwtAuthMiddleware {
      service: Rc::new(service),
      jwt_config: self.jwt_config.clone(),
    })
  }
}

pub struct JwtAuthMiddleware<S> {
  service: Rc<S>,
  jwt_config: JwtSetting,
}

impl<S> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<body::BoxBody>, Error = Error> + 'static,
{
  type Response = ServiceResponse<body::BoxBody>;

  type Error = Error;

  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let svc = Rc::clone(&self.service);

    Box::pin(async move {
      if let Some(cookie) = req.cookie("auth") {
        let jwt_util = JwtUtil::new(&self.jwt_config);
        if let Ok(claims) = jwt_util.decode_token(cookie.value()) {
          req.extensions_mut().insert(claims.sub);
          return svc.call(req).await;
        }
      }
      Ok(req.into_response(HttpResponse::Unauthorized().finish()))
    })
  }
}
