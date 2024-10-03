use crate::config::job_config::{DuplicateEntryPolicy, OnDuplicateEntry};
use crate::grpc::result_collecting_service::ResultCollectingService;

use mongodb::bson::{doc, Bson, DateTime as BsonDateTime, Document};
use mongodb::{error::Error as MongoError, Collection};
use protos::gamayun::{EmptyResponse, JobResultWithMapOnly, MapResult};
use std::collections::HashMap;
use tonic::{Response, Status};
use tracing::{error, info};

impl ResultCollectingService {
    /// Processes results for a job that contains map-only data and handles storing them
    /// in MongoDB based on the duplicate entry policy of the job.
    ///
    /// # Arguments
    ///
    /// * `job_result` - The job result containing the name and a list of map results.
    ///
    /// # Returns
    ///
    /// `Result<Response<EmptyResponse>, Status>` - Returns an empty response on success,
    /// or a `Status` error if processing or storing fails.
    pub async fn handle_result_map_only(
        &self,
        job_result: JobResultWithMapOnly,
    ) -> Result<Response<EmptyResponse>, Status> {
        // Extract job name and results
        let job_name = job_result.name;
        let results = job_result.results;

        // Match the job config using the new function
        let job_config = self.match_job_config(&job_name)?;

        // Get the MongoDB collection
        let collection = self
            .app_context
            .mongo_client
            .database(self.app_context.mongo_db_name.as_str())
            .collection(&job_name);

        // Check for duplicate entry policy, default to TrackChanges
        let duplicate_policy = job_config
            .duplicate_entry_policy
            .clone()
            .unwrap_or_else(|| DuplicateEntryPolicy {
                unique_ids: vec![],
                on_duplicate_entry: OnDuplicateEntry::TrackChanges,
            });

        // Extract tags from job_config
        let tags = job_config.tags.clone();

        // Handle each map result based on the duplicate entry policy
        for map_result in results {
            Self::handle_single_result(
                &job_name,
                &collection,
                &duplicate_policy,
                &tags,
                map_result,
            )
            .await?;
        }

        info!("Successfully processed all results for job: {}", job_name);
        Ok(Response::new(EmptyResponse {}))
    }

    /// Processes a single map result for a job and stores it in MongoDB based on the
    /// provided duplicate entry policy.
    ///
    /// # Arguments
    ///
    /// * `job_name` - The name of the job for which the result is being processed.
    /// * `collection` - The MongoDB collection to store the result.
    /// * `duplicate_policy` - The policy that defines how duplicate entries should be handled.
    /// * `tags` - Tags associated with the job.
    /// * `map_result` - The individual map result to be processed.
    ///
    /// # Returns
    ///
    /// `Result<(), Status>` - Returns `Ok(())` on success or a `Status` error if processing fails.
    async fn handle_single_result(
        job_name: &String,
        collection: &Collection<Document>,
        duplicate_policy: &DuplicateEntryPolicy,
        tags: &Vec<String>,
        map_result: MapResult,
    ) -> Result<(), Status> {
        let map: HashMap<String, String> = map_result.map_result;

        // Current timestamp to add to new documents
        let current_time = BsonDateTime::now();

        let mut doc = map
            .clone()
            .into_iter()
            .fold(Document::new(), |mut acc, (k, v)| {
                acc.insert(k, v);
                acc
            });

        // Add created_at, updated_at, and tags fields
        doc.insert("gamayun_created_at", current_time);
        doc.insert("gamayun_updated_at", current_time);
        doc.insert(
            "gamayun_tags",
            Bson::Array(tags.iter().map(|tag| Bson::String(tag.clone())).collect()),
        );

        // Build the filter for unique ID fields
        let filter = Self::build_unique_filter(&map, &duplicate_policy);

        // Store the document based on the duplicate policy
        Self::store_based_on_duplicate_policy(
            &job_name,
            &collection,
            duplicate_policy.clone(),
            doc,
            filter,
        )
        .await
        .map_err(|e| {
            error!("MongoDB operation failed: {}", e);
            Status::internal(format!("MongoDB error: {}", e))
        })?;
        Ok(())
    }

    /// Stores a document in MongoDB based on the duplicate entry policy for the job.
    ///
    /// # Arguments
    ///
    /// * `job_name` - The name of the job for which the document is being stored.
    /// * `collection` - The MongoDB collection to store the document.
    /// * `duplicate_policy` - The policy defining how duplicate entries should be handled.
    /// * `doc` - The document to be stored.
    /// * `filter` - A filter to identify existing documents matching the unique IDs.
    ///
    /// # Returns
    ///
    /// `Result<(), MongoError>` - Returns `Ok(())` on successful storage or a `MongoError`
    /// if any MongoDB operation fails.
    async fn store_based_on_duplicate_policy(
        job_name: &String,
        collection: &Collection<Document>,
        duplicate_policy: DuplicateEntryPolicy,
        doc: Document,
        filter: Document,
    ) -> Result<(), MongoError> {
        match duplicate_policy.on_duplicate_entry {
            OnDuplicateEntry::IgnoreNew => {
                Self::handle_ignore_new_policy(job_name, collection, doc, filter).await
            }
            OnDuplicateEntry::Overwrite => {
                Self::handle_overwrite_policy(job_name, collection, doc, filter).await
            }
            OnDuplicateEntry::TrackChanges => {
                Self::handle_track_changes_policy(job_name, collection, doc, filter).await
            }
        }
    }

    /// Handles the `IgnoreNew` duplicate entry policy by skipping new documents if
    /// a duplicate is found, updating only `gamayun_updated_at` in the existing document.
    ///
    /// # Arguments
    ///
    /// * `job_name` - The name of the job.
    /// * `collection` - The MongoDB collection.
    /// * `doc` - The new document to potentially store.
    /// * `filter` - The filter to find existing duplicates.
    ///
    /// # Returns
    ///
    /// `Result<(), MongoError>` - Returns `Ok(())` on successful storage or a `MongoError`
    /// if a MongoDB operation fails.
    async fn handle_ignore_new_policy(
        job_name: &String,
        collection: &Collection<Document>,
        doc: Document,
        filter: Document,
    ) -> Result<(), MongoError> {
        // Skip inserting if a document with the same unique ID already exists
        if collection.find_one(filter.clone()).await?.is_none() {
            collection.insert_one(doc).await?;
        } else {
            // Update `gamayun_updated_at` field only
            let current_time = BsonDateTime::now();
            let update = doc! {
                "$set": { "gamayun_updated_at": current_time }
            };
            collection.update_one(filter, update).await?;
            info!(
                "Duplicate found, ignoring new entry as per policy for job: {}",
                job_name
            );
        }
        Ok(())
    }

    /// Handles the `Overwrite` duplicate entry policy by replacing existing documents
    /// while preserving the `gamayun_created_at` field.
    ///
    /// # Arguments
    ///
    /// * `job_name` - The name of the job.
    /// * `collection` - The MongoDB collection.
    /// * `doc` - The new document to store.
    /// * `filter` - The filter to find existing duplicates.
    ///
    /// # Returns
    ///
    /// `Result<(), MongoError>` - Returns `Ok(())` on successful storage or a `MongoError`
    /// if a MongoDB operation fails.
    async fn handle_overwrite_policy(
        job_name: &String,
        collection: &Collection<Document>,
        mut doc: Document,
        filter: Document,
    ) -> Result<(), MongoError> {
        // Find existing document and keep its `gamayun_created_at`
        if let Some(existing_doc) = collection.find_one(filter.clone()).await? {
            if let Some(gamayun_created_at) = existing_doc.get("gamayun_created_at").cloned() {
                doc.insert("gamayun_created_at", gamayun_created_at);
            }
        }

        // Replace the existing document with the updated fields
        collection.replace_one(filter, doc).await?;
        info!(
            "Duplicate found, overwriting entry as per policy for job: {}",
            job_name
        );
        Ok(())
    }

    /// Handles the `TrackChanges` duplicate entry policy by creating a new document
    /// containing only the fields that have changed since the last stored document.
    ///
    /// # Arguments
    ///
    /// * `job_name` - The name of the job.
    /// * `collection` - The MongoDB collection.
    /// * `doc` - The new document to compare and store.
    /// * `filter` - The filter to find existing duplicates.
    ///
    /// # Returns
    ///
    /// `Result<(), MongoError>` - Returns `Ok(())` on successful storage or a `MongoError`
    /// if a MongoDB operation fails.
    async fn handle_track_changes_policy(
        job_name: &String,
        collection: &Collection<Document>,
        doc: Document,
        filter: Document,
    ) -> Result<(), MongoError> {
        // Find the latest document matching the filter
        if let Some(existing_doc) = collection.find_one(filter.clone()).await? {
            // Calculate the changed fields
            let mut changed_fields = Document::new();
            for (key, new_value) in &doc {
                if let Some(existing_value) = existing_doc.get(key) {
                    // Compare existing value with new value
                    if existing_value != new_value {
                        changed_fields.insert(key.clone(), new_value.clone());
                    }
                } else {
                    // New field that didn't exist before
                    changed_fields.insert(key.clone(), new_value.clone());
                }
            }

            // Add timestamps to changed document
            let current_time = BsonDateTime::now();
            changed_fields.insert("gamayun_created_at", current_time);
            changed_fields.insert("gamayun_updated_at", current_time);

            // Insert only the changed fields
            collection.insert_one(changed_fields).await?;
        } else {
            // If no existing document, insert the full document
            collection.insert_one(doc).await?;
        }

        info!(
            "Tracking changes, inserted new entry as per policy for job: {}",
            job_name
        );
        Ok(())
    }
}
