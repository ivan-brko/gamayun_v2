mod common_utils;
mod impl_empty_result_handling;
mod impl_error_handling;
mod impl_result_handling;

use crate::init::AppContext;
use protos::gamayun::result_reporting_service_server::ResultReportingService;
use protos::gamayun::{EmptyResponse, JobError, JobResult, RunInformation};
use tonic::{Request, Response, Status};
use tracing::error;
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
        match job_result.run_information {
            Some(run_information) => {
                // Create a span with `name` and `runId` added to the tracing context

                let span = tracing::info_span!(
                    "report_result",
                    name = %run_information.job_name,
                    run_id = %run_information.run_id
                );

                // Instrument the future to use the span for subsequent logs
                self.handle_result(job_result.results, run_information)
                    .instrument(span)
                    .await
            }
            None => {
                error!("Received result for job with no runId");
                Err(Status::invalid_argument("RunInformation is required"))
            }
        }
    }

    async fn report_no_result(
        &self,
        request: Request<RunInformation>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let run_information = request.into_inner();
        let span = tracing::info_span!(
            "report_no_result",
            name = %run_information.job_name,
            run_id = %run_information.run_id
        );

        // Instrument the future to use the span for subsequent logs
        self.handle_reported_no_result(run_information)
            .instrument(span)
            .await
    }

    async fn report_error(
        &self,
        request: Request<JobError>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let job_error = request.into_inner();
        match job_error.run_information {
            Some(run_information) => {
                // Create a span with `name` and `runId` added to the tracing context
                let span = tracing::info_span!(
                    "report_error",
                    name = %run_information.job_name,
                    run_id = %run_information.run_id
                );

                // Instrument the future to use the span for subsequent logs
                self.handle_error(job_error.error, run_information)
                    .instrument(span)
                    .await
            }
            None => {
                error!("Received error for job with no runId");
                Err(Status::invalid_argument("RunInformation is required"))
            }
        }
    }
}
