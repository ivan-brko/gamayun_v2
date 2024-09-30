use actix_web::Responder;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

#[tracing::instrument]
async fn validate_request() {
    // this is new
    info!("Validating request");
    sleep(Duration::from_millis(50)).await;
    // this is new
    info!("Successfully validated request");
}

// this function simulates storing to db
// this is new
#[tracing::instrument]
async fn store_to_db(_name: &str) {
    // this is new
    info!("Storing to DB");
    sleep(Duration::from_millis(150)).await;
    // this is new
    info!("Successfully stored to DB");
}

// this is the function that does all the logic for the request handling
// this is new
#[tracing::instrument]
pub(crate) async fn inner_greet(name: &str) -> impl Responder {
    // this is new
    info!("Handling request");
    validate_request().await;
    store_to_db(name).await;
    // this is new
    info!("Successfully handled the request");
    format!("Hello {name}, you have beed stored in the DB!")
}
