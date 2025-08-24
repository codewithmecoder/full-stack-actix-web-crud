use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use tiberius::{Client, Config};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub type DbClient = Client<Compat<TcpStream>>;
pub type DbPool = Arc<Mutex<Vec<DbClient>>>;

pub struct DbManager {
  pools: Arc<Mutex<HashMap<String, DbPool>>>,
}

impl DbManager {
  pub fn new() -> Self {
    Self {
      pools: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  /// Initialize a connection pool for a given name (if not exists)
  pub async fn init_pool(&self, pool_name: &str, conn_str: &str, pool_size: u32) -> Result<()> {
    let mut pools = self.pools.lock().await;
    if pools.contains_key(pool_name) {
      return Ok(()); // already initialized
    }

    let mut connections = Vec::with_capacity(pool_size as usize);

    for _ in 0..pool_size {
      let mut config =
        Config::from_ado_string(conn_str).or_else(|_| Config::from_jdbc_string(conn_str))?;
      config.trust_cert();

      let tcp = TcpStream::connect(config.get_addr()).await?;
      tcp.set_nodelay(true)?;

      let client = Client::connect(config, tcp.compat_write()).await?;
      connections.push(client);
    }

    pools.insert(pool_name.to_string(), Arc::new(Mutex::new(connections)));
    Ok(())
  }

  /// Get a pooled client wrapped in a guard (auto-return when dropped)
  pub async fn get_client(&self, pool_name: &str) -> Result<PooledClient> {
    let pools = self.pools.lock().await;
    let pool = pools
      .get(pool_name)
      .ok_or_else(|| anyhow::anyhow!("Pool `{}` not found", pool_name))?;

    let mut pool_guard = pool.lock().await;
    let client = pool_guard
      .pop()
      .ok_or_else(|| anyhow::anyhow!("Pool `{}` is empty", pool_name))?;

    Ok(PooledClient {
      name: pool_name.to_string(),
      client: Some(client),
      manager: self.clone(),
    })
  }

  /// Return a client back to the pool
  async fn return_client(&self, pool_name: &str, client: DbClient) -> Result<()> {
    let pools = self.pools.lock().await;
    let pool = pools
      .get(pool_name)
      .ok_or_else(|| anyhow::anyhow!("Pool `{}` not found", pool_name))?;

    let mut pool_guard = pool.lock().await;
    pool_guard.push(client);
    Ok(())
  }
}

// Needed so DbManager can be cloned (Arc inside makes this cheap)
impl Clone for DbManager {
  fn clone(&self) -> Self {
    Self {
      pools: self.pools.clone(),
    }
  }
}

/// Wrapper that auto-returns client when dropped
pub struct PooledClient {
  pub name: String,
  pub client: Option<DbClient>,
  pub manager: DbManager,
}
impl PooledClient {
  pub fn client(&mut self) -> &mut DbClient {
    self.client.as_mut().unwrap()
  }
}
impl Drop for PooledClient {
  fn drop(&mut self) {
    if let Some(client) = self.client.take() {
      let name = self.name.clone();
      let manager = self.manager.clone();
      // Return client asynchronously in background
      tokio::spawn(async move {
        let _ = manager.return_client(&name, client).await;
      });
    }
  }
}
