use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use tiberius::Row;
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

impl User {
  pub fn from_row(row: &Row) -> Self {
    let naive_created_at: NaiveDateTime = row
      .try_get::<NaiveDateTime, &str>("created_at")
      .expect("Failed to get created_at")
      .unwrap_or_default();
    let created_at: DateTime<Utc> =
      DateTime::<Utc>::from_naive_utc_and_offset(naive_created_at, Utc);

    let naive_updated_at: NaiveDateTime = row
      .try_get::<NaiveDateTime, &str>("updated_at")
      .expect("Failed to get updated_at")
      .unwrap_or_default();
    let updated_at: DateTime<Utc> =
      DateTime::<Utc>::from_naive_utc_and_offset(naive_updated_at, Utc);
    Self {
      id: row
        .try_get::<i32, &str>("id")
        .expect("Failed to get id")
        .unwrap_or_default(), // fallback if null
      name: row
        .try_get::<&str, &str>("name")
        .expect("Failed to get name")
        .unwrap_or_default()
        .to_string(),
      email: row
        .try_get::<&str, &str>("email")
        .expect("Failed to get email")
        .unwrap_or_default()
        .to_string(),
      user_name: row
        .try_get::<&str, &str>("user_name")
        .expect("Failed to get user_name")
        .unwrap_or_default()
        .to_string(),
      password: row
        .try_get::<&str, &str>("password")
        .expect("Failed to get password")
        .unwrap_or_default()
        .to_string(),
      role: UserRole::from_str(
        row
          .try_get::<&str, &str>("role")
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
