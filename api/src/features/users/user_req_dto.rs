use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct UserRegisterReqDto {
  pub user_name: String,
  pub password: String,
  pub email: String,
  pub name: String,
}
