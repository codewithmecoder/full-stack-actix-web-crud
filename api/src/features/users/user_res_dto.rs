use serde::{Deserialize, Serialize};

use crate::features::users::user_entity::User;

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
