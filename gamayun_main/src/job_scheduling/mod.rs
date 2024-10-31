pub mod config_reload;
pub mod scheduled_job_tracking_service;

use crate::config::job_config::JobConfig;
use crate::job_scheduling::scheduled_job_tracking_service::ScheduledJobTrackingService;
use crate::notification::composite_notification_sender::CompositeNotificationSender;
use anyhow::{Context, Result};
use chrono::Utc;
use grizzly_scheduler::scheduler::Scheduler;
use mongodb::bson::uuid;
use std::process::Command;
use tracing::{error, info};
use tracing_futures::Instrument;

pub const SCHEDULED_GAMAYUN_JOB_CATEGORY: &str = "SCHEDULED_GAMAYUN_JOB";

pub fn start_background_job_reporting_check(
    scheduler: Scheduler<Utc>,
    notification_sender: CompositeNotificationSender,
) -> ScheduledJobTrackingService {
    ScheduledJobTrackingService::new(scheduler, notification_sender)
}

pub fn schedule_jobs_from_config(
    scheduler: Scheduler<Utc>,
    scheduled_job_tracking_service: ScheduledJobTrackingService,
    config_root: String,
) -> Result<Vec<JobConfig>> {
    let job_configs = JobConfig::load_configs_from_directory(&config_root)
        .context("Failed to load job configurations")?;

    for job_config in &job_configs {
        schedule_single_job(
            scheduler.clone(),
            job_config.clone(),
            scheduled_job_tracking_service.clone(),
        );
    }

    Ok(job_configs)
}

fn schedule_single_job(
    scheduler: Scheduler<Utc>,
    job_config: JobConfig,
    scheduled_job_tracking_service: ScheduledJobTrackingService,
) {
    info!("Scheduling job: {}", job_config.name);

    let path_to_executable = job_config.path_to_executable.clone();
    let job_name = job_config.name.clone();
    let arguments = job_config.arguments.clone();
    let result_wait_timeout_millis = job_config.result_wait_timeout_millis.unwrap_or(10_000); // default to 10 seconds

    // Schedule the job to run based on the cron schedule
    scheduler
        .schedule_sequential_job(
            &job_config.cron_string,
            Some(job_config.name),
            Some(SCHEDULED_GAMAYUN_JOB_CATEGORY.to_string()),
            job_config
                .random_trigger_offset_seconds
                .map(|offset| chrono::Duration::seconds(offset)),
            move || {
                let path_to_executable = path_to_executable.clone();
                let job_name = job_name.clone();
                let arguments = arguments.clone();
                let scheduled_job_tracking_service = scheduled_job_tracking_service.clone();
                run_single_job(
                    path_to_executable,
                    job_name,
                    arguments,
                    result_wait_timeout_millis,
                    scheduled_job_tracking_service,
                )
            },
        )
        .expect("Failed to schedule job");
}

async fn run_single_job(
    path_to_executable: String,
    job_name: String,
    arguments: Vec<String>,
    result_wait_timeout_millis: i64,
    scheduled_job_tracking_service: ScheduledJobTrackingService,
) {
    let unique_id = uuid::Uuid::new().to_string();
    let span = tracing::info_span!(
        "run_single_job",
        job_name = %job_name,
        unique_id = %unique_id
    );

    async move {
        info!("Executing job");

        // Start the OS task
        match Command::new(&path_to_executable)
            .env("GAMAYUN_JOB_NAME", &job_name)
            .env("GAMAYUN_JOB_UNIQUE_ID", &unique_id)
            .args(arguments)
            .spawn()
        {
            Ok(child) => {
                info!("Job {} started with PID {}", &job_name, child.id());
                scheduled_job_tracking_service
                    .add_job(
                        job_name.clone(),
                        unique_id,
                        chrono::Duration::milliseconds(result_wait_timeout_millis),
                    )
                    .await;
            }
            Err(e) => {
                error!("Failed to start job {}: {:?}", job_name, e);
            }
        }
    }
    .instrument(span)
    .await
}
