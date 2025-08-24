use crate::{
  app_state::AppState,
  features::users::{user_entity::User, user_req_dto::UserRegisterReqDto},
  repos::sql_repo::{CommandType, DbPool, SqlRepo},
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

  async fn create_connection(&self) -> Result<DbPool> {
    SqlRepo::create_connection(
      &self.app_state.config.database.conn_str,
      self.app_state.config.database.pool_size,
    )
    .await
  }

  pub async fn create(&mut self, user: &UserRegisterReqDto) -> Result<()> {
    let params: Vec<&dyn ToSql> = vec![&user.name, &user.user_name, &user.email, &user.password];

    let mut client_pool = self.create_connection().await?.lock().await.pop().unwrap();

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
    let mut client_pool = self.create_connection().await?.lock().await.pop().unwrap();

    let user = SqlRepo::execute_command_single_query(
      &mut client_pool,
      "[dbo].[select_user]",
      &[&id],
      CommandType::StoreProcedure,
      |row| User::from_row(row),
    )
    .await?;
    Ok(user.transpose()?)
  }
}
