use crate::config::job_config::{DuplicateEntryPolicy, JobConfig};
use crate::grpc::result_collecting_service::ResultCollectingService;
use mongodb::bson::Document;
use std::collections::HashMap;
use tonic::Status;
use tracing::error;

impl ResultCollectingService {
    pub fn match_job_config(&self, job_name: &str) -> std::result::Result<&JobConfig, Status> {
        // Find the job config based on the job name
        match self
            .app_context
            .job_configs
            .iter()
            .find(|config| config.name == job_name)
        {
            Some(config) => Ok(config),
            None => {
                error!("No job config found for job name: {}", job_name);
                Err(Status::not_found(format!(
                    "No job config found for job name: {}",
                    job_name
                )))
            }
        }
    }

    pub fn build_unique_filter(
        map: &HashMap<String, String>,
        policy: &DuplicateEntryPolicy,
    ) -> Document {
        // Build a filter document based on unique_ids fields
        let mut filter = Document::new();
        for unique_field in &policy.unique_ids {
            if let Some(value) = map.get(unique_field) {
                filter.insert(unique_field, value.clone());
            }
        }
        filter
    }
}
