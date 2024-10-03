use crate::config::job_config::JobConfig;
use anyhow::{Context, Result};
use grizzly_scheduler::scheduler::Scheduler;
use std::env;
use std::future::Future;
use std::process::Command;
use chrono::Utc;
use tracing::{error, info};

pub fn schedule_jobs_from_config(scheduler: Scheduler<chrono::Utc>) -> Result<()> {
    let config_root = env::var("GAMAYUN_CONFIGURATION_ROOT")
        .context("GAMAYUN_CONFIGURATION_ROOT environment variable is not set")?;

    let job_configs = JobConfig::load_configs_from_directory(&config_root)
        .context("Failed to load job configurations")?;

    for job_config in job_configs {
        schedule_single_job(scheduler.clone(), job_config);
    }

    Ok(())
}

fn schedule_single_job(scheduler: Scheduler<Utc>, job_config: JobConfig) {
    info!("Scheduling job: {}", job_config.name);

    let path_to_executable = job_config.path_to_executable.clone();
    let job_name = job_config.name.clone();
    let arguments = job_config.arguments.clone();

    // Schedule the job to run based on the cron schedule
    scheduler
        .schedule_sequential_job(
            &job_config.cron_string,
            Some(job_config.name),
            job_config
                .random_trigger_offset_seconds
                .map(|offset| chrono::Duration::seconds(offset)),
            move || run_single_job(path_to_executable.clone(), job_name.clone(), arguments.clone()),
        )
        .expect("Failed to schedule job");
}

fn run_single_job(path_to_executable: String, job_name: String, arguments: Vec<String>) -> impl Future<Output=()> + Sized {
    info!("Executing job: {}", &job_name);

    // Start the OS task
    match Command::new(&path_to_executable)
        .args(arguments)
        .spawn()
    {
        
        Ok(child) => {
            info!("Job {} started with PID {}", &job_name, child.id());

            // todo: we can use the lower part later to somehow async wait for
            // the process to complete and use that information to know when to
            // stop waiting for the results of the job

            // Optionally wait for the process to complete
            // match child.wait() {
            //     Ok(status) => {
            //         if status.success() {
            //             info!("Job {} finished successfully", &job_name);
            //         } else {
            //             info!("Job {} finished with status {}", &job_name, status);
            //         }
            //     }
            //     Err(e) => {
            //         info!("Failed to wait for job {}: {:?}", job_name, e);
            //     }
            // }
        }
        Err(e) => {
            error!("Failed to start job {}: {:?}", job_name, e);
        }

    }

    // Return an empty future
    async {  }
}