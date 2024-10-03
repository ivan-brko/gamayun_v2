use chrono::Utc;
use std::env;
use tracing::info;

use crate::config::job_config::JobConfig;
use crate::job_scheduling::schedule_jobs_from_config;
use grizzly_scheduler::scheduler::Scheduler;
use mongodb::Client;

mod mongo;
mod observability;

#[derive(Clone)]
pub struct AppContext {
    pub mongo_client: Client,
    pub scheduler: Scheduler<Utc>,
    pub job_configs: Vec<JobConfig>,
    pub mongo_db_name: String,
}

pub async fn initialize() -> AppContext {
    info!("Initializing app");
    dotenv::dotenv().ok();
    observability::initialize_tracing_subscriber();

    // Initialize MongoDB client
    let mongo_client = mongo::initialize_mongo_client()
        .await
        .expect("Failed to initialize MongoDB client");

    let mongo_db_name = get_mongo_db_name();

    // Initialize the scheduler
    let scheduler = grizzly_scheduler::scheduler::Scheduler::new_in_utc();

    // Schedule jobs from config
    let job_configs = schedule_jobs_from_config(scheduler.clone())
        .expect("Failed to schedule jobs from config during app init");

    scheduler.start().expect("Failed to start scheduler");

    info!("App Initialized");

    // Return the app context
    AppContext {
        mongo_client,
        scheduler,
        job_configs,
        mongo_db_name,
    }
}

// Get MongoDB database name from environment variable or default to "gamayun"
fn get_mongo_db_name() -> String {
    env::var("GAMAYUN_MONGO_DB_NAME").unwrap_or_else(|_| "gamayun".to_string())
}
