use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct LoginResDto {
  pub token: String,
}

impl Default for LoginResDto {
  fn default() -> Self {
    LoginResDto {
      token: "".to_string(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub sub: i32,    // Subject (e.g., user ID)
  pub exp: usize,  // Expiration time (Unix timestamp)
  pub iss: String, // Issuer
  pub aud: String, // Audience
}

// --- Request Dto --- //
#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct LoginReqDto {
  pub user_name: String,
  pub password: String,
}
