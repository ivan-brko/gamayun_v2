use chrono::Utc;
use tracing::info;

use crate::job_scheduling::schedule_jobs_from_config;
use grizzly_scheduler::scheduler::Scheduler;

mod observability;

pub fn initialize() -> Scheduler<Utc> {
    info!("Initializing app");
    observability::initialize_tracing_subscriber();

    let scheduler = grizzly_scheduler::scheduler::Scheduler::new_in_utc();

    schedule_jobs_from_config(scheduler.clone())
        .expect("Failed to schedule jobs from config during app init");

    info!("App Initialized");

    scheduler
}
