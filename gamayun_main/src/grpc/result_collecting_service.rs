use protos::gamayun::result_server::Result;
use protos::gamayun::{
    EmptyResponse, JobError, JobResultWithMapAndStrings, JobResultWithMapOnly,
    JobResultWithRawStringsOnly,
};
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct ResultCollectingService {
    mongo_client: mongodb::Client,
}

impl ResultCollectingService {
    pub fn new(mongo_client: mongodb::Client) -> Self {
        Self { mongo_client }
    }
}

#[tonic::async_trait]
impl Result for ResultCollectingService {
    async fn report_result_with_raw_strings_only(
        &self,
        request: Request<JobResultWithRawStringsOnly>,
    ) -> std::result::Result<Response<EmptyResponse>, Status> {
        let job_result = request.into_inner();
        info!("Received raw strings only result: {:?}", job_result);
        Ok(Response::new(EmptyResponse {}))
    }

    async fn report_result_with_map_only(
        &self,
        request: Request<JobResultWithMapOnly>,
    ) -> std::result::Result<Response<EmptyResponse>, Status> {
        let job_result = request.into_inner();
        info!("Received map only result: {:?}", job_result);
        Ok(Response::new(EmptyResponse {}))
    }

    async fn report_result_with_map_and_strings(
        &self,
        request: Request<JobResultWithMapAndStrings>,
    ) -> std::result::Result<Response<EmptyResponse>, Status> {
        let job_result = request.into_inner();
        info!("Received map and strings result: {:?}", job_result);
        Ok(Response::new(EmptyResponse {}))
    }

    async fn report_error(
        &self,
        request: Request<JobError>,
    ) -> std::result::Result<Response<EmptyResponse>, Status> {
        let job_error = request.into_inner();
        error!("Received job error: {:?}", job_error);
        Ok(Response::new(EmptyResponse {}))
    }
}
