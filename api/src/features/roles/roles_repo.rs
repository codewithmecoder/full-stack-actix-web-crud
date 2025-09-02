use crate::{
  app_state::AppState,
  features::roles::roles_entity::{RoleEntity, UserRolesEntity},
  repos::{
    sql_pool_manager::PooledClient,
    sql_repo::{CommandType, SqlRepo},
  },
};

use anyhow::Result;
use tiberius::ToSql;

pub struct RoleRepo<'a> {
  pub app_state: &'a AppState,
}

impl<'a> RoleRepo<'a> {
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

  pub async fn create_role(&mut self, role: &RoleEntity) -> Result<u64> {
    let mut client_pool = self.get_client().await;

    let description = role.description.clone().unwrap_or_default();
    let params: Vec<&dyn ToSql> = vec![&role.name, &description];
    let result = SqlRepo::execute_command_none_query(
      &mut client_pool,
      "[dbo].[create_role]",
      &params,
      CommandType::StoreProcedure,
    )
    .await?;
    Ok(result)
  }

  pub async fn update_role(&mut self, role: &RoleEntity) -> Result<u64> {
    let mut client_pool = self.get_client().await;

    let description = role.description.clone().unwrap_or_default();
    let params: Vec<&dyn ToSql> = vec![&role.id, &role.name, &description];
    let result = SqlRepo::execute_command_none_query(
      &mut client_pool,
      "[dbo].[update_role]",
      &params,
      CommandType::StoreProcedure,
    )
    .await?;
    Ok(result)
  }

  pub async fn get_by_name(&mut self, name: &str) -> Result<Option<RoleEntity>> {
    let mut client_pool = self.get_client().await;

    let role = SqlRepo::execute_command_single_query(
      &mut client_pool,
      "[dbo].[select_role_by_name]",
      &[&name],
      CommandType::StoreProcedure,
      |row| RoleEntity::from(row),
    )
    .await?;
    Ok(role)
  }

  pub async fn get_by_id(&mut self, id: i32) -> Result<Option<RoleEntity>> {
    let mut client_pool = self.get_client().await;

    let role = SqlRepo::execute_command_single_query(
      &mut client_pool,
      "[dbo].[select_role_by_id]",
      &[&id],
      CommandType::StoreProcedure,
      |row| RoleEntity::from(row),
    )
    .await?;
    Ok(role)
  }

  pub async fn get_user_roles(&mut self, user_id: i32) -> Result<Vec<UserRolesEntity>> {
    let mut client_pool = self.get_client().await;

    let user_roles = SqlRepo::execute_command_query(
      &mut client_pool,
      "[dbo].[select_user_role]",
      &[&user_id],
      CommandType::StoreProcedure,
      |row| UserRolesEntity::from(row),
    )
    .await?;
    Ok(user_roles)
  }
}
