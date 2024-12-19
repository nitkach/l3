use anyhow::Result;
use log::info;
use std::net::Ipv4Addr;
use tokio::net::TcpListener;

mod app;
mod error;
mod model;
mod repository;

pub async fn run(addresses: Addresses) -> Result<()> {
    let listener = TcpListener::bind(addresses.app).await?;
    info!("Binded address: {}:{}", addresses.app.0, addresses.app.1);

    let shared_state = repository::Repository::init(&addresses.redis).await?;
    let router = app::initialize_router(shared_state);

    axum::serve(listener, router).await?;

    Ok(())
}

#[derive(Debug)]
pub struct Addresses {
    pub app: (Ipv4Addr, u16),
    pub redis: String,
}
