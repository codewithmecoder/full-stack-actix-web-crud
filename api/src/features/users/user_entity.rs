use chrono::{DateTime, NaiveDateTime, Utc};
use domner_tech_sql_client::pool_manager::DbRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
  pub id: i32,
  pub user_name: String,
  pub name: String,
  pub password: String,
  pub email: String,
  pub role: UserRole,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<&DbRow<'_>> for User {
  fn from(row: &DbRow<'_>) -> Self {
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
      email: row
        .get_mssql::<&str>("email")
        .expect("Failed to get email")
        .unwrap_or_default()
        .to_string(),
      user_name: row
        .get_mssql::<&str>("user_name")
        .expect("Failed to get user_name")
        .unwrap_or_default()
        .to_string(),
      password: row
        .get_mssql::<&str>("password")
        .expect("Failed to get password")
        .unwrap_or_default()
        .to_string(),
      role: UserRole::from_str(
        row
          .get_mssql::<&str>("role")
          .expect("Failed to get role")
          .unwrap(),
      ),
      created_at: created_at,
      updated_at: updated_at,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub enum UserRole {
  Admin,
  Moderator,
  User,
}

impl UserRole {
  pub fn to_str(&self) -> &str {
    match self {
      UserRole::Admin => "admin",
      UserRole::Moderator => "moderator",
      UserRole::User => "user",
    }
  }

  pub fn from_str(s: &str) -> Self {
    match s {
      "admin" => UserRole::Admin,
      "moderator" => UserRole::Moderator,
      _ => UserRole::User,
    }
  }
}
