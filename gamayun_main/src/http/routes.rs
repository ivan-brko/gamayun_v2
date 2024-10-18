use crate::http::app_config_reload_handler::reload_job_config;
use crate::http::version_retriever::retrieve_version;
use actix_web::{web, Scope};

pub(crate) fn assemble_routes() -> Scope {
    web::scope("/api/v1")
        .service(reload_job_config)
        .service(retrieve_version)
}
