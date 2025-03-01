use std::net::Ipv4Addr;

use anyhow::Result;
use log::info;
use url::Url;

mod app;
mod error;
mod model;
mod repository;

pub async fn run() -> Result<()> {
    let config = Config::new()?;
    info!("Config created");

    let app = app::App::initialize(config).await?;
    info!("App initialized");

    info!("Serving app");
    app.run().await?;

    Ok(())
}

pub(crate) struct Config {
    address: (Ipv4Addr, u16),
    database_config: deadpool_postgres::Config,
}

impl Config {
    fn new() -> Result<Self> {
        let address = {
            let host = match std::env::var("HOST") {
                Ok(host) => host.parse::<Ipv4Addr>()?,
                Err(_) => Ipv4Addr::new(127, 0, 0, 1),
            };
            let port = match std::env::var("PORT") {
                Ok(port) => port.parse::<u16>()?,
                Err(_) => 3000,
            };
            (host, port)
        };

        let database_config = {
            let pg_user = std::env::var("PGUSER")?;
            let pg_password = std::env::var("PGPASSWORD")?;
            let pg_db = std::env::var("PGDATABASE")?;
            let pg_host = std::env::var("PGHOST")?;
            let pg_port = std::env::var("PGPORT")?.parse::<u16>()?;

            info!("Created config for connection to database: 'postgres://{pg_user}:{pg_password}@{pg_host}:{pg_port}/{pg_db}'");

            let mut database_config = deadpool_postgres::Config::new();

            database_config.user = Some(pg_user);
            database_config.password = Some(pg_password);
            database_config.dbname = Some(pg_db);
            database_config.host = Some(pg_host);
            database_config.port = Some(pg_port);

            database_config
        };

        Ok(Self {
            address,
            database_config,
        })
    }
}
