use crate::init::AppContext;
use crate::job_scheduling::config_reload::handle_config_reload_request;
use crate::notification::NotificationSender;
use actix_web::{post, web, HttpResponse, Responder};
use tracing::{error, info};

#[post("/jobs/config/reload")]
pub(super) async fn reload_job_config(app_context: web::Data<AppContext>) -> impl Responder {
    info!("Received request to reload job configuration");
    handle_jobs_config_reload_request(app_context).await
}

#[tracing::instrument(skip(app_context))]
pub(crate) async fn handle_jobs_config_reload_request(
    app_context: web::Data<AppContext>,
) -> impl Responder {
    info!("Reloading job configuration");
    match handle_config_reload_request(app_context.get_ref().clone()).await {
        Ok(_) => {
            info!("Job configuration reloaded successfully");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!("Failed to reload job configuration: {:?}", e);
            app_context
                .notification_sender
                .notify(
                    "Gamayun Job Configuration Reload Failure".to_string(),
                    format!("Failed to reload job configuration: {:?}", e),
                )
                .await;
            HttpResponse::InternalServerError()
                .body("Failed to reload job configuration".to_string())
        }
    }
}
