use crate::grpc::result_collecting_service::ResultCollectingService;
use crate::notification::NotificationSender;
use protos::gamayun::{EmptyResponse, JobError};
use tonic::{Response, Status};
use tracing::{error, instrument};

impl ResultCollectingService {
    #[instrument(skip(self))]
    pub async fn handle_error(
        &self,
        job_error: JobError,
    ) -> Result<Response<EmptyResponse>, Status> {
        // Log the error
        error!("Received job error: {:?}", job_error);

        let run_id = job_error.run_id.clone();

        self.app_context
            .background_job_completion_scheduler
            .report_result_returned(&run_id)
            .await;

        self.app_context
            .notification_sender
            .notify(
                format!("Gamayun Error for job {}", job_error.name),
                format!(
                    "The following error was reported for job {} with run id {}: \n{}",
                    job_error.name, job_error.run_id, job_error.error
                ),
            )
            .await;

        // Return an empty response
        Ok(Response::new(EmptyResponse {}))
    }
}
