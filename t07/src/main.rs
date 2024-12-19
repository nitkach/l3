use anyhow::Result;
use log::warn;
use std::{net::Ipv4Addr, process::ExitCode};
use t07::Addresses;

#[tokio::main]
async fn main() -> ExitCode {
    setup_logging();

    if let Err(err) = dotenvy::dotenv() {
        warn!(".env file: {err}");
    }

    let addresses = match get_address() {
        Ok(address) => address,
        Err(err) => {
            eprintln!("{err:?}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = t07::run(addresses).await {
        eprintln!("{err:?}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn get_address() -> Result<Addresses> {
    let app = (
        std::env::var("HOST")?.parse::<Ipv4Addr>()?,
        std::env::var("PORT")?.parse::<u16>()?,
    );

    let redis = {
        let host = std::env::var("REDIS_HOST")?;
        let port = std::env::var("REDIS_PORT")?.parse::<u16>()?;
        format!("redis://{host}:{port}/")
    };

    Ok(Addresses { app, redis })
}

fn setup_logging() {
    env_logger::init();
}
