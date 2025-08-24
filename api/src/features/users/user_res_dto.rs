use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserDto {
  pub id: i32,
  pub user_name: String,
  pub name: String,
  pub email: String,
}
