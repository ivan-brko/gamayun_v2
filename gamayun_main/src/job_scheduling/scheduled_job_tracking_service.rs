use crate::notification::composite_notification_sender::CompositeNotificationSender;
use crate::notification::NotificationSender;
use chrono::{DateTime, Duration, Utc};
use grizzly_scheduler::scheduler::Scheduler;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct Job {
    pub name: String,
    pub run_id: String,
    pub valid_until: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ScheduledJobTrackingService {
    jobs: Arc<Mutex<HashMap<String, Job>>>,
    notification_sender: CompositeNotificationSender,
}

impl ScheduledJobTrackingService {
    pub fn new(
        scheduler: Scheduler<Utc>,
        notification_sender: CompositeNotificationSender,
    ) -> Self {
        let service = ScheduledJobTrackingService {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            notification_sender,
        };

        let notification_sender = service.notification_sender.clone();

        let jobs_clone = service.jobs.clone();
        scheduler
            .schedule_sequential_job(
                "*/10 * * * *", // run the job every 10 minutes
                Some("Overdue Job Checker".to_string()),
                None,
                Some(Duration::seconds(2)),
                move || {
                    let jobs = jobs_clone.clone();
                    let notification_sender = notification_sender.clone();
                    async move {
                        info!("Checking for overdue jobs.");
                        let mut jobs = jobs.lock().await; // Use `await` with the async mutex
                        let now = Utc::now();
                        let overdue_jobs: Vec<(String, String)> = jobs
                            .iter()
                            .filter(|(_, job)| job.valid_until < now)
                            .map(|(run_id, job)| (run_id.clone(), job.name.clone()))
                            .collect();

                        for (run_id, job_name) in overdue_jobs {
                            notification_sender
                                .notify(
                                    format!("Gamayun Overdue Job for {}", job_name),
                                    format!(
                                        "Job with name {} and  run ID {} is overdue.",
                                        job_name, run_id
                                    ),
                                )
                                .await;
                            error!(
                                "Error: Job with name {} and  run ID {} is overdue.",
                                job_name, run_id
                            );
                            jobs.remove(&run_id);
                        }
                    }
                },
            )
            .unwrap();

        service
    }

    pub async fn add_job(&self, name: String, run_id: String, duration: Duration) {
        let valid_until = Utc::now() + duration;
        let job = Job {
            name,
            run_id: run_id.clone(),
            valid_until,
        };
        let mut jobs = self.jobs.lock().await;
        jobs.insert(run_id, job);
    }

    pub async fn report_result_returned(&self, run_id: &String) {
        let mut jobs = self.jobs.lock().await;
        if jobs.remove(run_id).is_none() {
            error!("Error: Job with run ID {} not found.", run_id);
        } else {
            info!(
                "Successfully reported result for job with run ID {}.",
                run_id
            );
        }
    }

    pub async fn remove_all_jobs(&self) {
        let mut jobs = self.jobs.lock().await;
        jobs.clear();
        info!("All jobs have been removed.");
    }
}
