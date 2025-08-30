use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AppSetting {
  pub rust_log: String,
  pub server: ServerSetting,
  pub database: DatabaseSetting,
  pub jwt: JwtSetting,
  pub cookie: CookieSetting,
}

#[derive(Deserialize, Clone)]
pub struct ServerSetting {
  pub host: String,
  pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSetting {
  pub sql_server: DatabaseConnectionInfo,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConnectionInfo {
  pub conn_str: String,
  pub pool_size: u32,
  pub pool_name: String,
}

#[derive(Deserialize, Clone)]
pub struct JwtSetting {
  pub secret_key: String,
  pub expiration_minutes: usize,
  pub issuer: String,
  pub audience: String,
}

#[derive(Deserialize, Clone)]
pub struct CookieSetting {
  pub name: String,
}
