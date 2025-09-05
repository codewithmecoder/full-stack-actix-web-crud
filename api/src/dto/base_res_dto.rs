use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::commons::status_code_const::StatusCodeConst;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BaseResDto<T: ToSchema> {
  pub data: Option<T>,
  #[serde(default)]
  pub status: Status,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Status {
  #[serde(default)]
  pub message: String, // defaults to empty string
  #[serde(default = "default_code")]
  pub code: String, // defaults to "SUCCESS"
  pub status: u16,
}

// Default value for code
fn default_code() -> String {
  StatusCodeConst::SUCCESS.to_string()
}

// Implement Default for Status
impl Default for Status {
  fn default() -> Self {
    Status {
      message: String::new(),
      code: default_code(),
      status: 200,
    }
  }
}
// Implement Default for BaseResDto<T>
impl<T: ToSchema> Default for BaseResDto<T> {
  fn default() -> Self {
    BaseResDto {
      data: None,
      status: Status::default(),
    }
  }
}
