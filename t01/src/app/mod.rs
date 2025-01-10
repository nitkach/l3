use crate::repository;
use anyhow::Result;
use axum::Router;
use std::net::Ipv4Addr;
use tokio::net::TcpListener;

mod routes;

pub(crate) struct App {
    listener: TcpListener,
    router: Router,
}

impl App {
    pub(crate) async fn initialize() -> Result<Self> {
        let listener = Self::bind_listener().await?;

        let shared_state = repository::Repository::initialize().await?;

        let router = routes::initialize_router(shared_state);

        Ok(Self { listener, router })
    }

    pub(crate) async fn run(self) -> Result<()> {
        axum::serve(self.listener, self.router).await?;

        Ok(())
    }

    async fn bind_listener() -> Result<TcpListener> {
        let addr = {
            let host = std::env::var("HOST")?.parse::<Ipv4Addr>()?;
            let port = std::env::var("PORT")?.parse::<u16>()?;
            (host, port)
        };
        let listener = TcpListener::bind(addr).await?;
        Ok(listener)
    }
}
