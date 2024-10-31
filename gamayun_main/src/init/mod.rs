use anyhow::Context;
use chrono::Utc;
use std::env;
use tracing::{error, info};

use crate::config::app_config::initialize_app_config;
use crate::config::job_config::JobConfig;
use crate::job_scheduling::scheduled_job_tracking_service::ScheduledJobTrackingService;
use crate::job_scheduling::{schedule_jobs_from_config, start_background_job_reporting_check};
use crate::notification::composite_notification_sender::CompositeNotificationSender;
use crate::notification::NotificationSender;
use grizzly_scheduler::scheduler::Scheduler;
use mongodb::Client;

mod mongo;
mod notification_sender;
mod observability;

/// Struct representing the application context.
///
/// Holds various components required for the application, such as MongoDB client,
/// scheduler, job configurations, and the notification sender.
#[derive(Clone)]
pub struct AppContext {
    /// Application version
    pub app_version: String,
    /// Configuration root directory
    pub config_root: String,
    /// MongoDB client instance.
    pub mongo_client: Client,
    /// Background job completion scheduler.
    pub background_job_completion_scheduler: ScheduledJobTrackingService,
    /// Scheduler for job scheduling.
    pub scheduler: Scheduler<Utc>,
    /// Job configurations loaded from the config.
    pub job_configs: Vec<JobConfig>,
    /// Name of the MongoDB database.
    pub mongo_db_name: String,
    /// Composite notification sender used to send notifications.
    pub notification_sender: CompositeNotificationSender,
}

/// Initializes the first stage of the application.
///
/// This stage sets up observability and initializes the notification sender.
///
/// # Returns
///
/// A `Result` containing the `CompositeNotificationSender` or an error if initialization fails.
async fn init_notification_and_logging(
    app_version: String,
    config_root: String,
) -> Result<CompositeNotificationSender, Box<dyn std::error::Error>> {
    observability::initialize_tracing_subscriber(app_version);
    info!("Initializing app configuration");
    let app_config = initialize_app_config(config_root.clone())?;
    info!("App configuration initialized");

    let notification_sender = notification_sender::initialize_notification_sender(app_config);
    info!("Notification sender initialized");

    Ok(notification_sender)
}

/// Initializes the second stage of the application.
///
/// This stage initializes the MongoDB client, scheduler, and job configurations.
///
/// # Arguments
///
/// * `notification_sender` - A `CompositeNotificationSender` instance used for notifications.
///
/// # Returns
///
/// A `Result` containing the `AppContext` or an error if initialization fails.
async fn init_other_services(
    notification_sender: CompositeNotificationSender,
    app_version: String,
    config_root: String,
) -> Result<AppContext, Box<dyn std::error::Error>> {
    // Initialize MongoDB client
    let (mongo_client, mongo_db_name) = mongo::initialize_mongo_client().await?;

    // Initialize the scheduler
    let scheduler = grizzly_scheduler::scheduler::Scheduler::new_in_utc();

    let background_job_completion_scheduler =
        start_background_job_reporting_check(scheduler.clone(), notification_sender.clone());

    // Schedule jobs from config
    let job_configs = schedule_jobs_from_config(
        scheduler.clone(),
        background_job_completion_scheduler.clone(),
        config_root.clone(),
    )?;

    scheduler.start()?;

    info!("App Initialized");

    // Return the app context
    Ok(AppContext {
        app_version,
        config_root,
        mongo_client,
        background_job_completion_scheduler,
        scheduler,
        job_configs,
        mongo_db_name,
        notification_sender,
    })
}

/// Initializes the entire application context.
///
/// This function calls `init_stage_one` and `init_stage_two` to initialize all components
/// required for the application. If any stage fails, it logs the error, sends a notification
/// (if possible), and then panics.
///
/// # Returns
///
/// An `AppContext` instance representing the initialized application context.
pub async fn initialize() -> AppContext {
    dotenv::dotenv().ok();
    // Get the app version from the Cargo.toml file
    let app_version = env!("CARGO_PKG_VERSION").to_string();
    let config_root = env::var("GAMAYUN_CONFIGURATION_ROOT")
        .context("GAMAYUN_CONFIGURATION_ROOT environment variable is not set")
        .unwrap(); //we want to kill the app if this is not set

    match init_notification_and_logging(app_version.clone(), config_root.clone()).await {
        Ok(notification_sender) => {
            match init_other_services(notification_sender.clone(), app_version, config_root).await {
                Ok(context) => context,
                Err(e) => {
                    error!("Initialization failed: {}", e);
                    notification_sender
                        .notify("Initialization failed".to_string(), e.to_string())
                        .await;
                    panic!("Initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Initialization failed: {}", e);
            panic!("Initialization failed: {}", e);
        }
    }
}
