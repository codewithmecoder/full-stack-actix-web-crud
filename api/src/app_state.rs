use crate::app_settings::AppSetting;

use anyhow::Result;
use domner_tech_sql_client::pool_manager::DbManager;

#[derive(Clone)]
pub struct AppState {
  pub config: AppSetting,
  pub db_manager: DbManager,
}
impl AppState {
  // Load config from file manually
  pub async fn load_setting(path: &str) -> Result<Self> {
    let file = std::fs::File::open(path).expect(&format!("Failed to open config file: {}", path));
    let config: AppSetting = serde_json::from_reader(file).expect("Failed to parse JSON config");

    match Self::init_db_manager(&mut config.clone()).await {
      Ok(db_manager) => Ok(Self { config, db_manager }),
      Err(e) => Err(e),
    }
  }

  // Initialize the database manager with connection pools
  async fn init_db_manager(setting: &mut AppSetting) -> Result<DbManager> {
    let db_manager = DbManager::new();

    // Initialize the SQL connection pool
    db_manager
      .init_pool(
        setting.database.sql_server.pool_name.as_str(),
        &setting.database.sql_server.conn_str,
        setting.database.sql_server.pool_size,
      )
      .await?;
    Ok(db_manager)
  }
}
