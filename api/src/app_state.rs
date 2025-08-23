use crate::{
  app_settings::AppSetting,
  repos::sql_repo::{DbPool, SqlRepo},
};

use anyhow::{Error, Ok, Result};

#[derive(Clone)]
pub struct AppState {
  pub config: AppSetting,
  pub db_pool: DbPool,
}
impl AppState {
  // Load config from file manually
  pub async fn from_file(path: &str) -> Result<Self, Error> {
    let file = std::fs::File::open(path).expect(&format!("Failed to open config file: {}", path));
    let config: AppSetting = serde_json::from_reader(file).expect("Failed to parse JSON config");

    let client =
      SqlRepo::create_connection(&config.database.url, config.database.pool_size).await?;

    Ok(AppState {
      config,
      db_pool: client,
    })
  }
}
