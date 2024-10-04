mod common_utils;
mod impl_result_maps_only;

use crate::init::AppContext;
use protos::gamayun::result_server::Result;
use protos::gamayun::{EmptyResponse, JobError, JobResultWithMapOnly};
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument, Span};
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
impl Result for ResultCollectingService {
    async fn report_result_with_map_only(
        &self,
        request: Request<JobResultWithMapOnly>,
    ) -> std::result::Result<Response<EmptyResponse>, Status> {
        let job_result = request.into_inner();

        // Create a span with `name` and `runId` added to the tracing context
        let span = tracing::info_span!(
            "report_result_with_map_only",
            name = %job_result.name,
            run_id = %job_result.run_id
        );

        // Log that we received the result with the span context
        info!(parent: &span, "Received map only result: {:?}", job_result);

        // Instrument the future to use the span for subsequent logs
        self.handle_result_map_only(job_result)
            .instrument(span)
            .await
    }
    async fn report_error(
        &self,
        request: Request<JobError>,
    ) -> std::result::Result<Response<EmptyResponse>, Status> {
        let job_error = request.into_inner();
        error!("Received job error: {:?}", job_error);
        Ok(Response::new(EmptyResponse {}))
    }
}
