use crate::grpc::result_collecting_service::ResultCollectingService;
use crate::init::AppContext;
use anyhow::{Context, Result};
use protos::gamayun::result_reporting_service_server::ResultReportingServiceServer;
use std::env;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tracing::info;

mod result_collecting_service;

pub async fn run_grpc_server(
    app_context: AppContext,
    shutdown_token: CancellationToken,
) -> Result<()> {
    // Read gRPC address from environment variable or use default
    let addr = env::var("GAMAYUN_GRPC_ADDR")
        .unwrap_or_else(|_| "[::1]:16656".to_string())
        .parse()
        .context("Failed to parse GAMAYUN_GRPC_ADDR")?;

    let result_service = ResultCollectingService::new(app_context);

    info!("ResultService listening on {}", addr);

    Server::builder()
        .add_service(ResultReportingServiceServer::new(result_service))
        .serve_with_shutdown(addr, shutdown_token.cancelled())
        .await
        .context("gRPC server error")?;

    Ok(())
}
