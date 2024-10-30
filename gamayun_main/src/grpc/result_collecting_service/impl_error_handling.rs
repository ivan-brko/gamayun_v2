use crate::grpc::result_collecting_service::ResultCollectingService;
use crate::notification::NotificationSender;
use protos::gamayun::{EmptyResponse, RunInformation};
use tonic::{Response, Status};
use tracing::{error, instrument};

impl ResultCollectingService {
    /// Handles an error that occurred during a job's execution.
    ///
    /// # Arguments
    ///
    /// * `error` - A `String` describing the error that occurred.
    /// * `run_information` - Information about the job run, including `run_id` and `job_name`.
    ///
    /// # Returns
    ///
    /// `Result<Response<EmptyResponse>, Status>` - Returns an empty response on success,
    /// or a `Status` error if handling the error fails.
    #[instrument(skip(self))]
    pub async fn handle_error(
        &self,
        error: String,
        run_information: RunInformation,
    ) -> Result<Response<EmptyResponse>, Status> {
        // Log the error
        error!("Received job error: {:?}", error);

        let run_id = run_information.run_id.clone();

        self.app_context
            .background_job_completion_scheduler
            .report_result_returned(&run_id)
            .await;

        self.app_context
            .notification_sender
            .notify(
                format!("Gamayun Error for job {}", run_information.job_name),
                format!(
                    "The following error was reported for job {} with run id {}: \n{}",
                    run_information.job_name, run_information.run_id, error
                ),
            )
            .await;

        // Return an empty response
        Ok(Response::new(EmptyResponse {}))
    }
}
