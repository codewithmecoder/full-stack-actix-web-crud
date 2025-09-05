use actix_web::{HttpResponse, Responder, get};

use crate::dto::base_res_dto::Status;

#[utoipa::path(
    get,
    path = "/api/v1/healthz",
    tag = "Health Checker Endpoint",
    responses(
        (status = 200, description= "Authenticated User", body = Status),
    )
)]
#[get("/healthz")]
pub async fn health_checker_handler() -> impl Responder {
  HttpResponse::Ok().json(Status::success())
}
