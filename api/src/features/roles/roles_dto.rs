use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::features::roles::roles_entity::{RoleEntity, UserRoleEntity, UserRolesEntity};

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct UserRoleDto {
  pub id: i32,
  pub user_id: i32,
  pub role_id: i32,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct RoleDto {
  pub id: i32,
  pub name: String,
  pub description: Option<String>,
}

#[derive(Deserialize, Clone, ToSchema)]
pub struct CreateRoleReqDto {
  pub name: String,
  pub description: Option<String>,
}

#[derive(Deserialize, Clone, ToSchema)]
pub struct UpdateRoleReqDto {
  pub id: i32,
  pub name: String,
  pub description: Option<String>,
}

#[derive(Deserialize, Clone, ToSchema)]
pub struct GetUserRolesReqDto {
  pub user_id: i32,
}

#[derive(Deserialize, Clone, ToSchema)]
pub struct AssignUserRoleReqDto {
  pub user_id: i32,
  pub role_id: i32,
}

impl From<UserRoleEntity> for UserRoleDto {
  fn from(value: UserRoleEntity) -> Self {
    Self {
      id: value.id,
      user_id: value.user_id,
      role_id: value.role_id,
    }
  }
}

impl From<&RoleEntity> for RoleDto {
  fn from(value: &RoleEntity) -> Self {
    Self {
      id: value.id,
      name: value.name.clone(),
      description: value.description.clone(),
    }
  }
}

// ---------- Response Dto --------- //

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct UserRolesResDto {
  pub role_id: i32,
  pub role_name: String,
  pub is_in_role: bool,
}

impl From<&UserRolesEntity> for UserRolesResDto {
  fn from(value: &UserRolesEntity) -> Self {
    Self {
      role_id: value.role_id,
      role_name: value.role_name.clone(),
      is_in_role: value.is_in_role,
    }
  }
}
