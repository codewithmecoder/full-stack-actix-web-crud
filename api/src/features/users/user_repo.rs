use crate::{
  app_state::AppState,
  features::users::{
    user_dto::{UserDto, UserRegisterReqDto},
    user_entity::User,
  },
  repos::{
    sql_pool_manager::PooledClient,
    sql_repo::{CommandType, SqlRepo},
  },
  utils::password_hashing::PasswordHashing,
};

use anyhow::Result;
use tiberius::ToSql;

pub struct UserRepo<'a> {
  pub app_state: &'a AppState,
}

impl<'a> UserRepo<'a> {
  pub fn new(app_state: &'a AppState) -> Self {
    Self { app_state }
  }

  async fn get_client(&self) -> PooledClient {
    match self
      .app_state
      .db_manager
      .get_client(&self.app_state.config.database.sql_server.pool_name)
      .await
    {
      Ok(client) => client,
      Err(e) => panic!("Failed to get DB client: {}", e),
    }
  }

  pub async fn create(&mut self, user: &UserRegisterReqDto) -> Result<()> {
    let user_existed = self.get_by_username(&user.user_name).await?;

    if user_existed.is_some() {
      return Err(anyhow::anyhow!("Username already exists"));
    }

    // Hash the password before storing
    let hashed_password = PasswordHashing::hash_password(&user.password)
      .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

    let params: Vec<&dyn ToSql> = vec![
      &user.name,
      &user.user_name,
      &user.email,
      &hashed_password,
      &user.role,
    ];

    let mut client_pool = self.get_client().await;

    SqlRepo::execute_command_none_query(
      &mut client_pool,
      "[dbo].[create_user]",
      &params,
      CommandType::StoreProcedure,
    )
    .await?;
    Ok(())
  }

  pub async fn get_by_id(&mut self, id: i32) -> Result<Option<User>> {
    let mut client_pool = self.get_client().await;

    let user = SqlRepo::execute_command_single_query(
      &mut client_pool,
      "[dbo].[select_user]",
      &[&id],
      CommandType::StoreProcedure,
      |row| User::from_row(row),
    )
    .await?;
    Ok(user)
  }

  pub async fn get_by_username(&mut self, username: &str) -> Result<Option<User>> {
    let mut client_pool = self.get_client().await;

    let user = SqlRepo::execute_command_single_query(
      &mut client_pool,
      "[dbo].[select_user_by_user_name]",
      &[&username],
      CommandType::StoreProcedure,
      |row| User::from_row(row),
    )
    .await?;
    Ok(user)
  }

  pub async fn get_users(&mut self) -> Result<Vec<User>> {
    let mut client_pool = self.get_client().await;

    let users = SqlRepo::execute_command_query(
      &mut client_pool,
      "[dbo].[select_users]",
      &[],
      CommandType::StoreProcedure,
      |row| User::from_row(row),
    )
    .await?;
    Ok(users)
  }

  pub async fn update_user(&mut self, user: &UserDto) -> Result<u64> {
    let mut client_pool = self.get_client().await;

    let role = user.role.to_str();
    let params: Vec<&dyn ToSql> = vec![&user.name, &user.user_name, &user.email, &role];

    let result = SqlRepo::execute_command_none_query(
      &mut client_pool,
      "[dbo].[update_user]",
      &params,
      CommandType::StoreProcedure,
    )
    .await?;
    Ok(result)
  }
}
