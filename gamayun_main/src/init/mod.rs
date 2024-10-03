use chrono::Utc;
use tracing::info;

use crate::job_scheduling::schedule_jobs_from_config;
use grizzly_scheduler::scheduler::Scheduler;
use mongodb::Client;

mod mongo;
mod observability;

#[derive(Clone)]
pub struct AppContext {
    pub mongo_client: Client,
    pub scheduler: Scheduler<Utc>,
}

pub async fn initialize() -> AppContext {
    info!("Initializing app");
    dotenv::dotenv().ok();
    observability::initialize_tracing_subscriber();

    // Initialize MongoDB client
    let mongo_client = mongo::initialize_mongo_client()
        .await
        .expect("Failed to initialize MongoDB client");

    // Initialize the scheduler
    let scheduler = grizzly_scheduler::scheduler::Scheduler::new_in_utc();

    // Schedule jobs from config
    schedule_jobs_from_config(scheduler.clone())
        .expect("Failed to schedule jobs from config during app init");

    info!("App Initialized");

    // Return the app context
    AppContext {
        mongo_client,
        scheduler,
    }
}
