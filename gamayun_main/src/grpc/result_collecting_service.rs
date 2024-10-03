use tonic::{Request, Response, Status};
use protos::gamayun::result_server::Result;
use protos::gamayun::{
    EmptyResponse, JobError, JobResultWithMapAndStrings, JobResultWithMapOnly,
    JobResultWithRawStringsOnly,
};

#[derive(Default)]
pub struct ResultCollectingService {}

#[tonic::async_trait]
impl Result for ResultCollectingService {
    async fn report_result_with_raw_strings_only(&self, request: Request<JobResultWithRawStringsOnly>) -> std::result::Result<Response<EmptyResponse>, Status> {
            let job_result = request.into_inner();
            println!("Received raw strings only result: {:?}", job_result);
            Ok(Response::new(EmptyResponse {}))
    }

    async fn report_result_with_map_only(&self, request: Request<JobResultWithMapOnly>) -> std::result::Result<Response<EmptyResponse>, Status> {
            let job_result = request.into_inner();
            println!("Received map only result: {:?}", job_result);
            Ok(Response::new(EmptyResponse {}))
    }

    async fn report_result_with_map_and_strings(&self, request: Request<JobResultWithMapAndStrings>) -> std::result::Result<Response<EmptyResponse>, Status> {
            let job_result = request.into_inner();
            println!("Received map and strings result: {:?}", job_result);
            Ok(Response::new(EmptyResponse {}))
    }

    async fn report_error(&self, request: Request<JobError>) -> std::result::Result<Response<EmptyResponse>, Status> {
            let job_error = request.into_inner();
            eprintln!("Received job error: {:?}", job_error);
            Ok(Response::new(EmptyResponse {}))
    }
}

