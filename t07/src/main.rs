use anyhow::Result;
use std::{net::Ipv4Addr, process::ExitCode};

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = dotenvy::dotenv() {
        eprintln!(".env file: {err}");
        return ExitCode::FAILURE;
    }

    let address = match get_address() {
        Ok(address) => address,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = t07::run(address).await {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn get_address() -> Result<(Ipv4Addr, u16)> {
    let host = std::env::var("HOST")?.parse::<Ipv4Addr>()?;
    let port = std::env::var("PORT")?.parse::<u16>()?;
    Ok((host, port))
}
