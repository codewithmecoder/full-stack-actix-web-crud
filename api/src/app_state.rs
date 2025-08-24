use crate::app_settings::AppSetting;

use anyhow::{Error, Ok, Result};

#[derive(Clone)]
pub struct AppState {
  pub config: AppSetting,
}
impl AppState {
  // Load config from file manually
  pub async fn from_file(path: &str) -> Result<Self, Error> {
    let file = std::fs::File::open(path).expect(&format!("Failed to open config file: {}", path));
    let config: AppSetting = serde_json::from_reader(file).expect("Failed to parse JSON config");

    Ok(AppState { config })
  }
}
