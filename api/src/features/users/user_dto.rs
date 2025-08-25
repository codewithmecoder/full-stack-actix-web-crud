use crate::features::users::user_entity::User;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserDto {
  pub id: i32,
  pub user_name: String,
  pub name: String,
  pub email: String,
}

impl From<User> for UserDto {
  fn from(user: User) -> Self {
    UserDto {
      id: user.id,
      name: user.name,
      user_name: user.user_name,
      email: user.email,
    }
  }
}

// --- Request Dto --- //

#[derive(Deserialize, Serialize, Debug)]
pub struct UserRegisterReqDto {
  pub user_name: String,
  pub password: String,
  pub email: String,
  pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetUserByIdReqDto {
  pub id: i32,
}
