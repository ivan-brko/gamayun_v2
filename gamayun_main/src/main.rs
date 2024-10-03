use std::future::Future;
use tracing::info;
use anyhow::Result;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use crate::grpc::run_grpc_server;
use crate::http::run_actix_server;

mod config;
mod init;
mod job_scheduling;
mod grpc;
mod http;


#[tokio::main]
async fn main() -> Result<()> {
    init::initialize();

    info!("Starting the web server");

    let (shutdown_token, shutdown_future) = create_canceled_token();
    let http_server_future = run_actix_server(shutdown_token.clone());
    let grpc_server_future = run_grpc_server(shutdown_token.clone());

    // Run all futures concurrently
    tokio::select! {
        http_result = http_server_future => {
            info!("HTTP server has been shut down");
            http_result?;
        }
        grpc_result = grpc_server_future => {
            info!("gRPC server has been shut down");
            grpc_result?;
        }
        _ = shutdown_future => {
            info!("Shutdown signal received, servers will shut down via the cancellation token");
            // Shutdown signal received, servers will shut down via the cancellation token
        }
    }

    info!("Servers have been shut down");
    Ok(())
}

fn create_canceled_token() -> (CancellationToken, impl Future<Output=()> + Sized) {
    let shutdown_token = CancellationToken::new();
    let cloned_token = shutdown_token.clone();

    let shutdown_future = async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Received Ctrl+C, shutting down servers...");
        cloned_token.cancel();
    };

    (shutdown_token, shutdown_future)
}