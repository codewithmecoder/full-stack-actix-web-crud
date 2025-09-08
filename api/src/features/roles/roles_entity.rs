use chrono::{DateTime, NaiveDateTime, Utc};
use domner_tech_sql_client::pool_manager::DbRow;
use serde::{Deserialize, Serialize};

use crate::features::roles::roles_dto::{CreateRoleReqDto, RoleDto, UpdateRoleReqDto, UserRoleDto};

#[derive(Deserialize, Serialize, Clone)]
pub struct UserRoleEntity {
  pub id: i32,
  pub user_id: i32,
  pub role_id: i32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RoleEntity {
  pub id: i32,
  pub name: String,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UserRolesEntity {
  pub role_id: i32,
  pub role_name: String,
  pub is_in_role: bool,
}

impl From<&DbRow<'_>> for RoleEntity {
  fn from(row: &DbRow) -> Self {
    let naive_created_at: NaiveDateTime = row
      .get_mssql::<NaiveDateTime>("created_at")
      .expect("Failed to get created_at")
      .unwrap_or_default();
    let created_at: DateTime<Utc> =
      DateTime::<Utc>::from_naive_utc_and_offset(naive_created_at, Utc);

    let naive_updated_at: NaiveDateTime = row
      .get_mssql::<NaiveDateTime>("updated_at")
      .expect("Failed to get updated_at")
      .unwrap_or_default();
    let updated_at: DateTime<Utc> =
      DateTime::<Utc>::from_naive_utc_and_offset(naive_updated_at, Utc);

    Self {
      id: row
        .get_mssql::<i32>("id")
        .expect("Failed to get id")
        .unwrap_or_default(), // fallback if null
      name: row
        .get_mssql::<&str>("name")
        .expect("Failed to get name")
        .unwrap_or_default()
        .to_string(),
      description: row
        .get_mssql::<&str>("description")
        .unwrap_or_default()
        .map(|s| s.to_string()),
      created_at,
      updated_at,
    }
  }
}

impl From<RoleDto> for RoleEntity {
  fn from(value: RoleDto) -> Self {
    Self {
      id: value.id,
      name: value.name,
      description: value.description,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

impl From<&DbRow<'_>> for UserRoleEntity {
  fn from(value: &DbRow) -> Self {
    Self {
      id: value
        .get_mssql::<i32>("id")
        .expect("Failed to get id")
        .unwrap_or_default(),
      user_id: value
        .get_mssql::<i32>("user_id")
        .expect("Failed to get id")
        .unwrap_or_default(),
      role_id: value
        .get_mssql::<i32>("role_id")
        .expect("Failed to get id")
        .unwrap_or_default(),
    }
  }
}

impl From<UserRoleDto> for UserRoleEntity {
  fn from(value: UserRoleDto) -> Self {
    Self {
      id: value.id,
      user_id: value.user_id,
      role_id: value.role_id,
    }
  }
}

impl From<CreateRoleReqDto> for RoleEntity {
  fn from(value: CreateRoleReqDto) -> Self {
    Self {
      id: 0,
      name: value.name,
      description: value.description,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

impl From<UpdateRoleReqDto> for RoleEntity {
  fn from(value: UpdateRoleReqDto) -> Self {
    Self {
      id: value.id,
      name: value.name,
      description: value.description,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

impl From<&DbRow<'_>> for UserRolesEntity {
  fn from(row: &DbRow) -> Self {
    Self {
      role_id: row
        .get_mssql::<i32>("role_id")
        .expect("Failed to get role_id")
        .unwrap_or_default(), // fallback if null
      role_name: row
        .get_mssql::<&str>("role_name")
        .expect("Failed to get role_name")
        .unwrap_or_default()
        .to_string(),
      is_in_role: row
        .get_mssql::<bool>("is_in_role")
        .expect("Failed to get is_in_role")
        .unwrap_or_default(),
    }
  }
}
