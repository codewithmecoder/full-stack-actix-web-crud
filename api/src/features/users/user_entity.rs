use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tiberius::Row;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
  pub id: i32,
  pub user_name: String,
  pub name: String,
  pub password: String,
  pub email: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl User {
  pub fn from_row(row: &Row) -> Result<Self> {
    Ok(Self {
      id: row.try_get::<i32, &str>("id")?.unwrap_or_default(), // fallback if null
      name: row
        .try_get::<&str, &str>("name")?
        .unwrap_or_default()
        .to_string(),
      email: row
        .try_get::<&str, &str>("email")?
        .unwrap_or_default()
        .to_string(),
      user_name: row
        .try_get::<&str, &str>("user_name")?
        .unwrap_or_default()
        .to_string(),
      password: row
        .try_get::<&str, &str>("password")?
        .unwrap_or_default()
        .to_string(),
      created_at: row
        .try_get::<DateTime<Utc>, &str>("created_at")?
        .unwrap_or_else(Utc::now),
      updated_at: row
        .try_get::<DateTime<Utc>, &str>("updated_at")?
        .unwrap_or_else(Utc::now),
    })
  }
}
