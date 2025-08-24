use anyhow::{Ok, Result};
use tiberius::{Row, ToSql};

use crate::repos::sql_pool_manager::PooledClient;

#[derive(Debug, Clone, Copy)]
pub enum CommandType {
  Text,
  StoreProcedure,
  TableDirect,
}
impl CommandType {
  fn prefix(&self) -> &'static str {
    match self {
      CommandType::Text => "",
      CommandType::StoreProcedure => "EXEC ",
      CommandType::TableDirect => "SELECT * FROM ",
    }
  }
}

pub struct SqlRepo;

impl SqlRepo {
  /// Build query string with named parameters for stored procedure
  fn build_query_with_params(
    cmd_txt: &str,
    cmd_type: CommandType,
    params: &[&dyn ToSql],
  ) -> String {
    match cmd_type {
      CommandType::Text => cmd_txt.to_string(),
      CommandType::StoreProcedure => {
        let placeholders: Vec<String> = (0..params.len()).map(|i| format!("@P{}", i + 1)).collect();

        if placeholders.is_empty() {
          format!("{}{}", cmd_type.prefix(), cmd_txt)
        } else {
          format!(
            "{}{} {}",
            cmd_type.prefix(),
            cmd_txt,
            placeholders.join(", ")
          )
        }
      }
      CommandType::TableDirect => format!("{}{}", cmd_type.prefix(), cmd_txt),
    }
  }

  /// Execute non-query (INSERT/UPDATE/DELETE)
  /// Returns number of affected rows
  /// Example usage:
  /// let rows = SqlRepo::execute_command_none_query(&mut client, "UPDATE Users SET Name = @P1 WHERE Id = @P2", &[&"NewName", &1], CommandType::Text).await?;
  /// Ok(())
  pub async fn execute_command_none_query(
    pooled_client: &mut PooledClient,
    cmd_txt: &str,
    params: &[&dyn ToSql],
    cmd_type: CommandType,
  ) -> Result<u64> {
    let query = Self::build_query_with_params(cmd_txt, cmd_type, &params);
    let client = pooled_client.client();
    let rows = client.execute(&query, &params).await?;
    Ok(rows.total())
  }

  /// Execute multi entities as bulk insert
  /// table: table name
  /// columns: column names
  /// entities: array of entities, each entity is an array of values
  /// Example usage:
  /// let entities = vec![
  ///   vec![&1 as &dyn ToSql, &"Name1" as &dyn ToSql],
  ///   vec![&2 as &dyn ToSql, &"Name2" as &dyn ToSql],
  /// ];
  /// let rows = SqlRepo::execute_bulk_insert(&mut client, "Users", &["Id", "Name"], &entities).await?;
  /// Ok(())
  pub async fn execute_bulk_insert(
    pooled_client: &mut PooledClient,
    table: &str,
    columns: &[&str],
    entities: &[&[&dyn ToSql]],
  ) -> Result<u64> {
    let mut values = Vec::new();
    for entity in entities {
      let value_placeholders: Vec<String> =
        (0..entity.len()).map(|i| format!("@P{}", i + 1)).collect();
      values.push(format!("({})", value_placeholders.join(", ")));
    }

    let client = pooled_client.client();

    let query = format!(
      "INSERT INTO {} ({}) VALUES {}",
      table,
      columns.join(", "),
      values.join(", ")
    );
    let rows = client.execute(&query, &[]).await?;
    Ok(rows.total())
  }

  /// Execute a query returning multiple rows
  /// T is a closure that maps a Row to the desired type
  /// Returns a vector of T
  /// Example usage:
  /// let rows = SqlRepo::execute_command_query(&mut client, "SELECT * FROM
  /// Users", &[], CommandType::Text, |row| {
  ///   let id: i32 = row.get("Id");
  ///   let name: String = row.get("Name");
  ///   (id, name)
  /// }).await?;
  /// for (id, name) in rows {
  ///   println!("Id: {}, Name: {}", id, name);
  /// }
  /// Ok(())
  pub async fn execute_command_query<T>(
    pooled_client: &mut PooledClient,
    cmd_txt: &str,
    params: &[&dyn ToSql],
    cmd_type: CommandType,
    map_rows: impl Fn(&Row) -> T,
  ) -> Result<Vec<T>> {
    if cmd_txt.trim().is_empty() {
      return Ok(Vec::new());
    }

    let query = Self::build_query_with_params(cmd_txt, cmd_type, params);
    let client = pooled_client.client();
    // Execute query asynchronously
    let stream = client
      .query(query, params)
      .await
      .map_err(|e| anyhow::anyhow!("Failed to execute query '{}': {}", cmd_txt, e))?;

    let rows = stream.into_results().await?;

    let mut results: Vec<T> = Vec::new();
    for row in &rows[0] {
      results.push(map_rows(row));
    }
    Ok(results)
  }

  /// Execute a query returning a single row
  /// T is a closure that maps a Row to the desired type
  /// Returns None if no rows found
  pub async fn execute_command_single_query<T>(
    pooled_client: &mut PooledClient,
    cmd_txt: &str,
    params: &[&dyn ToSql],
    cmd_type: CommandType,
    map_row: impl Fn(&Row) -> T,
  ) -> Result<Option<T>> {
    let mut rows =
      Self::execute_command_query(pooled_client, cmd_txt, params, cmd_type, map_row).await?;
    Ok(rows.pop())
  }
}
