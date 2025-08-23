```Rust
use anyhow::Result;
use futures::TryStreamExt;
use std::collections::HashMap;
use std::net::TcpStream;
use tiberius::{Client, Config, AuthMethod, Row, types::SqlValue};
use tokio_util::compat::TokioAsyncWriteCompatExt;

/// Command type, like C# System.Data.CommandType
#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    Text,
    StoredProcedure,
    TableDirect,
}

impl CommandType {
    fn prefix(&self) -> &'static str {
        match self {
            CommandType::Text => "",
            CommandType::StoredProcedure => "EXEC ",
            CommandType::TableDirect => "SELECT * FROM ",
        }
    }
}

/// SQL Server repository using Tiberius
pub struct SqlSrvDbFactoryBaseRepo;

impl SqlSrvDbFactoryBaseRepo {
    /// Create a connection
    pub async fn create_connection(conn_str: &str) -> Result<Client<TcpStream>> {
        let mut config = conn_str.parse::<Config>()?;
        config.trust_cert();

        let tcp = tokio::net::TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        let client = Client::connect(config, tcp.compat_write()).await?;
        Ok(client)
    }

    /// Convert a HashMap of param name -> value into Tiberius SqlValue vector
    fn convert_params(params: &HashMap<&str, SqlValue<'_>>) -> Vec<SqlValue<'_>> {
        params.values().cloned().collect()
    }

    /// Build query string with named parameters for stored procedure
    fn build_query_with_params(command_text: &str, cmd_type: CommandType, params: &HashMap<&str, SqlValue<'_>>) -> String {
        match cmd_type {
            CommandType::Text => command_text.to_string(),
            CommandType::StoredProcedure => {
                let placeholders: Vec<String> = params.keys()
                    .enumerate()
                    .map(|(i, k)| format!("@{} = @P{}", k, i + 1))
                    .collect();
                format!("{}{} {}", cmd_type.prefix(), command_text, placeholders.join(", "))
            }
            CommandType::TableDirect => format!("{}{}", cmd_type.prefix(), command_text),
        }
    }

    /// Execute non-query (INSERT/UPDATE/DELETE)
    pub async fn execute_command_none_query(
        client: &mut Client<TcpStream>,
        command_text: &str,
        params: HashMap<&str, SqlValue<'_>>,
        cmd_type: CommandType,
    ) -> Result<u64> {
        let query = Self::build_query_with_params(command_text, cmd_type, &params);
        let sql_params = Self::convert_params(&params);
        let rows = client.execute(&query, &sql_params).await?;
        Ok(rows)
    }

    /// Execute multiple entities as bulk
    pub async fn execute_bulk_insert(
        client: &mut Client<TcpStream>,
        table: &str,
        columns: &[&str],
        entities: &[HashMap<&str, SqlValue<'_>>],
    ) -> Result<u64> {
        let mut values = Vec::new();
        for entity in entities {
            let row = columns
                .iter()
                .map(|col| format!("{}", entity.get(col).unwrap()))
                .collect::<Vec<_>>()
                .join(", ");
            values.push(format!("({})", row));
        }
        let query = format!(
            "INSERT INTO {} ({}) VALUES {}",
            table,
            columns.join(", "),
            values.join(", ")
        );
        let rows = client.execute(&query, &[]).await?;
        Ok(rows)
    }

    /// Execute a query returning multiple rows
    pub async fn execute_command_query<T>(
        client: &mut Client<TcpStream>,
        command_text: &str,
        params: HashMap<&str, SqlValue<'_>>,
        cmd_type: CommandType,
        map_row: impl Fn(&Row) -> T,
    ) -> Result<Vec<T>> {
        let query = Self::build_query_with_params(command_text, cmd_type, &params);
        let sql_params = Self::convert_params(&params);

        let mut stream = client.query(&query, &sql_params).await?;
        let mut results = Vec::new();

        while let Some(row) = stream.try_next().await? {
            results.push(map_row(&row));
        }

        Ok(results)
    }

    /// Execute a query returning a single row
    pub async fn execute_command_single_query<T>(
        client: &mut Client<TcpStream>,
        command_text: &str,
        params: HashMap<&str, SqlValue<'_>>,
        cmd_type: CommandType,
        map_row: impl Fn(&Row) -> T,
    ) -> Result<Option<T>> {
        let mut rows = Self::execute_command_query(client, command_text, params, cmd_type, map_row).await?;
        Ok(rows.pop())
    }
}
```

🔹 Usage Example: Stored Procedure with Output

```Rust
use env_logger::Env;
use std::collections::HashMap;
use tiberius::types::SqlValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let conn_str = "Server=localhost;Database=TestDb;User Id=sa;Password=YourPassword123;";
    let mut client = SqlSrvDbFactoryBaseRepo::create_connection(conn_str).await?;

    // Single insert via stored procedure
    let mut params = HashMap::new();
    params.insert("UserIdk", SqlValue::Int32(101));
    params.insert("AcceptTC", SqlValue::Bit(true));
    params.insert("IsBlocked", SqlValue::Bit(false));

    let rows_affected = SqlSrvDbFactoryBaseRepo::execute_command_none_query(
        &mut client,
        "[user].[InsertUser]",
        params,
        CommandType::StoredProcedure,
    ).await?;
    println!("Rows inserted: {}", rows_affected);

    // Query multiple rows
    let mut query_params = HashMap::new();
    query_params.insert("IsBlocked", SqlValue::Bit(false));

    let users = SqlSrvDbFactoryBaseRepo::execute_command_query(
        &mut client,
        "[user].[GetUsersByBlockedStatus]",
        query_params,
        CommandType::StoredProcedure,
        |row| {
            let user_id: i32 = row.get("UserIdk").unwrap();
            let accept_tc: bool = row.get("AcceptTC").unwrap();
            (user_id, accept_tc)
        },
    ).await?;

    println!("Users: {:?}", users);

    Ok(())
}
```

✅ Now you have:

- Single row query
- Multiple row query
- Bulk non-query insert/update/delete
- Stored procedure support
- Same parameter mapping

### For the Rust version I provided using Tiberius, here are all the main packages (crates) you need in your

```toml
[dependencies]
# Async runtime
tokio = { version = "1.38", features = ["full"] }

# SQL Server client
tiberius = { version = "0.12", features = ["chrono", "uuid"] }

# Tokio compatibility utilities for Tiberius
tokio-util = "0.9"

# Futures for async streams
futures = "0.3"

# Error handling (similar to exceptions in C#)
anyhow = "1.0"

# Logging
log = "0.4"
env_logger = "0.10"

# Optional: HashMap and collections
indexmap = "1.10"  # If you want ordered HashMap, optional                                         # For error handling
```

🔹 Notes:

- tokio is required because Tiberius is async.
- tiberius is the core SQL Server client.
- tokio-util is needed for .compat_write() so Tiberius can work with tokio::net::TcpStream.
- futures gives TryStreamExt for iterating query results.
- anyhow makes error handling convenient.
