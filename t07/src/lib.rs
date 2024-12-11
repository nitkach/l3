use anyhow::Result;
use std::net::Ipv4Addr;
use tokio::net::TcpListener;

mod app;
mod model;
mod repository;

pub async fn run(address: (Ipv4Addr, u16)) -> Result<()> {
    let listener = TcpListener::bind(address).await?;
    let shared_state = repository::Repository::init().await?;
    let router = app::initialize_router(shared_state);

    axum::serve(listener, router).await?;

    Ok(())
}
