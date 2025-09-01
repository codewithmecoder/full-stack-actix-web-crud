use core::fmt;

use actix_web::{HttpResponse, ResponseError, body};

use crate::{
  commons::status_code_const::StatusCodeConst,
  dto::base_res_dto::{BaseResDto, Status},
};

impl fmt::Display for Status {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", serde_json::to_string(&self).unwrap())
  }
}

#[derive(Debug, PartialEq)]
pub enum StatusMessage {
  Success,
  ServerError,
  NotFound(String),
  Unauthorized,
  PermissionDenied,
  UserNameExisted,
  Existed(String),
  UserNameExeedMaxLength(usize),
  WrongParams,
  DecodeTokenErr,
}

impl ToString for StatusMessage {
  fn to_string(&self) -> String {
    self.to_str().to_owned()
  }
}

impl Into<String> for StatusMessage {
  fn into(self) -> String {
    self.to_string()
  }
}

impl StatusMessage {
  pub fn to_str(&self) -> String {
    match self {
      StatusMessage::Success => "Success".to_string(),
      StatusMessage::ServerError => "Server error, Please try again later".to_string(),
      StatusMessage::NotFound(item_name) => format!("{} not found", item_name),
      StatusMessage::Unauthorized => "Invalid credentials".to_string(),
      StatusMessage::PermissionDenied => "Permission denied".to_string(),
      StatusMessage::UserNameExisted => "Username already existed".to_string(),
      StatusMessage::Existed(ex) => format!("{} already existed", ex),
      StatusMessage::UserNameExeedMaxLength(max_length) => {
        format!("Username cannot exceed {} characters", max_length)
      }
      StatusMessage::WrongParams => "Invalid input".to_string(),
      StatusMessage::DecodeTokenErr => "Decoded token failed".to_string(),
    }
  }
}

impl Status {
  // pub fn new(message: impl Into<String>, code: String, status: u16) -> Self {
  //   Status {
  //     message: message.into(),
  //     code: code,
  //     status: status,
  //   }
  // }

  pub fn success() -> Self {
    Status {
      status: 200,
      message: StatusMessage::Success.to_str(),
      code: StatusCodeConst::SUCCESS.to_string(),
    }
  }

  pub fn success_with_data<T>(data: T) -> BaseResDto<T> {
    BaseResDto {
      data: Some(data),
      status: Status {
        status: 200,
        message: StatusMessage::Success.to_str(),
        code: StatusCodeConst::SUCCESS.to_string(),
      },
    }
  }

  pub fn server_error(message: impl Into<String>) -> Self {
    Status {
      status: 500,
      message: message.into(),
      code: StatusCodeConst::SERVER_ERROR.to_string(),
    }
  }

  pub fn bad_request(message: impl Into<String>) -> Self {
    Status {
      status: 400,
      message: message.into(),
      code: StatusCodeConst::ERROR.to_string(),
    }
  }

  pub fn unauthorized(message: impl Into<String>) -> Self {
    Status {
      status: 401,
      message: message.into(),
      code: StatusCodeConst::UNAUTHORIZED.to_string(),
    }
  }

  pub fn token_missing() -> Self {
    Status {
      status: 401,
      message: "Unauthorized, token missing".into(),
      code: StatusCodeConst::TOKEN_MISSING.to_string(),
    }
  }

  pub fn uqique_constraint_voilation(message: impl Into<String>) -> Self {
    Status {
      status: 409,
      message: message.into(),
      code: StatusCodeConst::UQIQUE_CONSTRAINT.to_string(),
    }
  }

  pub fn not_found(message: impl Into<String>) -> Self {
    Status {
      status: 404,
      message: message.into(),
      code: StatusCodeConst::NOT_FOUND.to_string(),
    }
  }

  pub fn forbidden() -> Self {
    Status {
      status: 403,
      message: StatusMessage::PermissionDenied.to_str(),
      code: StatusCodeConst::FORBIDDEN.to_string(),
    }
  }

  pub fn into_http_response(self) -> HttpResponse {
    match self.status {
      500 => HttpResponse::InternalServerError().json(BaseResDto::<()> {
        data: None,
        status: self,
      }),
      403 => HttpResponse::Forbidden().json(BaseResDto::<()> {
        data: None,
        status: self,
      }),
      400 => HttpResponse::BadRequest().json(BaseResDto::<()> {
        data: None,
        status: self,
      }),
      404 => HttpResponse::NotFound().json(BaseResDto::<()> {
        data: None,
        status: self,
      }),
      401 => HttpResponse::Unauthorized().json(BaseResDto::<()> {
        data: None,
        status: self,
      }),
      409 => HttpResponse::Conflict().json(BaseResDto::<()> {
        data: None,
        status: self,
      }),
      _ => {
        eprintln!(
          "Warning: Missing pattern match. Converted status code {} to 500",
          self.status
        );
        HttpResponse::InternalServerError().json(BaseResDto::<()> {
          status: Status {
            message: StatusMessage::ServerError.into(),
            code: StatusCodeConst::SERVER_ERROR.to_string(),
            status: 500,
          },
          data: None,
        })
      }
    }
  }
}

impl std::error::Error for Status {}

impl ResponseError for Status {
  fn error_response(&self) -> HttpResponse<body::BoxBody> {
    let cloned = self.clone();
    cloned.into_http_response()
  }
}
