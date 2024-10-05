use crate::grpc::result_collecting_service::ResultCollectingService;
use crate::notification::NotificationSender;
use protos::gamayun::{EmptyResponse, JobError};
use tonic::{Response, Status};
use tracing::{error, instrument};

impl ResultCollectingService {
    #[instrument(skip(self))]
    pub async fn handle_error(
        &self,
        job_result: JobError,
    ) -> Result<Response<EmptyResponse>, Status> {
        // Log the error
        error!("Received job error: {:?}", job_result);
        self.app_context
            .notification_sender
            .notify(
                format!("Gamayun Error for job {}", job_result.name),
                format!(
                    "The following error was reported for job {} with run id {}: \n{}",
                    job_result.name, job_result.run_id, job_result.error
                ),
            )
            .await;

        // Return an empty response
        Ok(Response::new(EmptyResponse {}))
    }
}
