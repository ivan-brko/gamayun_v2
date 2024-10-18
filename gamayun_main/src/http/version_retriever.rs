use crate::init::AppContext;
use actix_web::{get, web, Responder, Result};
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
struct VersionResponse {
    version: String,
}

#[get("/version")]
pub(super) async fn retrieve_version(app_context: web::Data<AppContext>) -> Result<impl Responder> {
    info!("Received request to reload job configuration");
    Ok(web::Json(VersionResponse {
        version: app_context.app_version.clone(),
    }))
}
