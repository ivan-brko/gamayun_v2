mod common_utils;
mod impl_error_handling;
mod impl_result_handling;

use crate::init::AppContext;
use protos::gamayun::result_reporting_service_server::ResultReportingService;
use protos::gamayun::{EmptyResponse, JobError, JobResult};
use tonic::{Request, Response, Status};
use tracing_futures::Instrument;

pub struct ResultCollectingService {
    app_context: AppContext,
}

impl ResultCollectingService {
    pub fn new(app_context: AppContext) -> Self {
        Self { app_context }
    }
}

#[tonic::async_trait]
impl ResultReportingService for ResultCollectingService {
    async fn report_result(
        &self,
        request: Request<JobResult>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let job_result = request.into_inner();

        // Create a span with `name` and `runId` added to the tracing context
        let span = tracing::info_span!(
            "report_result",
            name = %job_result.name,
            run_id = %job_result.run_id
        );

        // Instrument the future to use the span for subsequent logs
        self.handle_result(job_result).instrument(span).await
    }
    async fn report_error(
        &self,
        request: Request<JobError>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let job_error = request.into_inner();
        // Create a span with `name` and `runId` added to the tracing context
        let span = tracing::info_span!(
            "report_error",
            name = %job_error.name,
            run_id = %job_error.run_id
        );

        // Instrument the future to use the span for subsequent logs
        self.handle_error(job_error).instrument(span).await
    }
}
