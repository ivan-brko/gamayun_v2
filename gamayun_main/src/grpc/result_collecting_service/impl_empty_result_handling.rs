use crate::grpc::result_collecting_service::ResultCollectingService;

use protos::gamayun::{EmptyResponse, RunInformation};
use tonic::{Response, Status};
use tracing::{info, instrument};
impl ResultCollectingService {
    /// Processes results for a job that contains no results. We still report those results to the
    /// collector to make sure we don't get warnings about missing results.
    ///
    /// # Arguments
    ///
    /// * `run_information` - Information about the job that got no results.
    ///
    /// # Returns
    ///
    /// `Result<Response<EmptyResponse>, Status>` - Returns an empty response on success,
    /// or a `Status` error if processing or storing fails.
    #[instrument(skip(self))]
    pub async fn handle_reported_no_result(
        &self,
        run_information: RunInformation,
    ) -> Result<Response<EmptyResponse>, Status> {
        // Extract job name and results
        let job_name = run_information.job_name;
        let run_id = run_information.run_id;

        self.app_context
            .background_job_completion_scheduler
            .report_result_returned(&run_id)
            .await;

        info!(
            "Successfully marked the no-results job as completed: {} and run id {}",
            job_name, run_id
        );
        Ok(Response::new(EmptyResponse {}))
    }
}
