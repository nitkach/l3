use anyhow::Result;
use redis::{aio::MultiplexedConnection, AsyncCommands};

#[derive(Clone)]
pub(crate) struct Repository {
    redis_conn: MultiplexedConnection,
}

impl Repository {
    pub(crate) async fn init() -> Result<Self> {
        let redis_hostname = std::env::var("REDIS_HOSTNAME")?;

        let client = redis::Client::open(redis_hostname)?;
        let redis_conn = client.get_multiplexed_tokio_connection().await?;

        Ok(Self { redis_conn })
    }

    pub(crate) async fn add_event(&self) -> Result<()> {
        todo!()
    }
}
