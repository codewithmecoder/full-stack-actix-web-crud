use crate::features::users::user_entity::{User, UserRole};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserDto {
  pub id: i32,
  pub user_name: String,
  pub name: String,
  pub email: String,
  pub role: UserRole,
}

impl From<User> for UserDto {
  fn from(user: User) -> Self {
    UserDto {
      id: user.id,
      name: user.name,
      user_name: user.user_name,
      email: user.email,
      role: user.role,
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
  pub role: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetUserByIdReqDto {
  pub id: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateUserReqDto {
  pub user_name: String,
  pub name: Option<String>,
  pub email: Option<String>,
  pub role: Option<String>,
}
