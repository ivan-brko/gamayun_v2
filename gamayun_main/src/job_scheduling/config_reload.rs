use crate::init::AppContext;
use crate::job_scheduling::{schedule_jobs_from_config, SCHEDULED_GAMAYUN_JOB_CATEGORY};
use tracing::{info, instrument};

#[instrument(skip(app_context))]
pub(crate) async fn handle_config_reload_request(app_context: AppContext) -> Result<(), String> {
    info!("Stopping all scheduled jobs");
    app_context
        .scheduler
        .stop_jobs_by_category(SCHEDULED_GAMAYUN_JOB_CATEGORY)
        .map_err(|e| format!("Failed to stop jobs by category: {:?}", e))?;

    info!("Removing all background job completion jobs");
    app_context
        .background_job_completion_scheduler
        .remove_all_jobs()
        .await;

    info!("Scheduling jobs from config");
    schedule_jobs_from_config(
        app_context.scheduler.clone(),
        app_context.background_job_completion_scheduler.clone(),
        app_context.config_root.clone(),
    )
    .map_err(|e| format!("Failed to schedule jobs from config: {:?}", e))?;

    Ok(())
}
