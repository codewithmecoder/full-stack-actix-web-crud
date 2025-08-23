use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ServerSetting {
  pub host: String,
  pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSetting {
  pub url: String,
  pub pool_size: u32,
}

#[derive(Deserialize, Clone)]
pub struct AppSetting {
  pub rust_log: String,
  pub server: ServerSetting,
  pub database: DatabaseSetting,
}
