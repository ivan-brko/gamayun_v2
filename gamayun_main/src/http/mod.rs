use crate::init::AppContext;
use actix_web::{post, web, App, HttpServer, Responder};
use anyhow::{Context, Result};
use std::env;
use std::net::ToSocketAddrs;
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_actix_web::TracingLogger;

mod app_config_reload_handler;

#[post("/jobs/config/reload")]
async fn reload_job_config(app_context: web::Data<AppContext>) -> impl Responder {
    info!("Received request to reload job configuration");
    app_config_reload_handler::handle_jobs_config_reload_request(app_context).await
}

pub async fn run_actix_server(
    app_context: AppContext,
    shutdown_token: CancellationToken,
) -> Result<()> {
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

    let state = web::Data::new(app_context);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(state.clone())
            .service(reload_job_config)
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
