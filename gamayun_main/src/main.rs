use crate::grpc::run_grpc_server;
use crate::http::run_actix_server;
use anyhow::Result;
use std::future::Future;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::info;

mod config;
mod grpc;
mod http;
mod init;
mod job_scheduling;
mod notification;

#[tokio::main]
async fn main() -> Result<()> {
    let app_context = init::initialize().await;

    info!("Starting the web server and gRPC server...");

    let (shutdown_token, shutdown_future) = create_cancellation_token();
    let http_server_future = run_actix_server(app_context.clone(), shutdown_token.clone());
    let grpc_server_future = run_grpc_server(app_context.clone(), shutdown_token.clone());

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

fn create_cancellation_token() -> (CancellationToken, impl Future<Output = ()> + Sized) {
    let shutdown_token = CancellationToken::new();
    let cloned_token = shutdown_token.clone();

    let shutdown_future = async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Received Ctrl+C, shutting down servers...");
        cloned_token.cancel();
    };

    (shutdown_token, shutdown_future)
}
