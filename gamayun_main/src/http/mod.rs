use actix_web::{get, web, App, HttpServer, Responder};
use tracing_actix_web::TracingLogger;
use anyhow::{Context, Result};
use tokio_util::sync::CancellationToken;
use tracing::info;
use std::env;
use std::net::ToSocketAddrs;

mod request_handling_logic;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    request_handling_logic::inner_greet(&name).await
}

pub async fn run_actix_server(shutdown_token: CancellationToken) -> Result<()> {
    let host = env::var("GAMAYUN_HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("GAMAYUN_HTTP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .context("Failed to parse GAMAYUN_HTTP_PORT")?;

    let addr = (host.as_str(), port)
        .to_socket_addrs()
        .context("Failed to resolve socket address")?
        .next()
        .context("No socket addresses yielded")?;

    info!("Starting Actix Web server on {}:{}", host, port);

    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(greet)
    })
        .bind(addr)
        .context("Failed to bind Actix Web server")?;

    let server_handle = server.run();

    tokio::select! {
        result = server_handle => {
            result.context("Actix Web server error")?;
        }
        _ = shutdown_token.cancelled() => {
            info!("Shutdown signal received, stopping Actix Web server...");
            // Perform any cleanup or graceful shutdown here if needed
        }
    }

    info!("Actix Web server has been shut down");
    Ok(())
}