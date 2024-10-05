use dotenv::dotenv;
use mongodb::{error::Result, Client};
use std::env;

pub async fn initialize_mongo_client() -> Result<(Client, String)> {
    // Load environment variables from a .env file if present
    dotenv().ok();

    // Read the connection string from an environment variable
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI environment variable is not set");

    // Create a MongoDB client
    let client = Client::with_uri_str(&mongo_uri).await?;

    let db_name = get_mongo_db_name();

    Ok((client, db_name))
}

// Get MongoDB database name from environment variable or default to "gamayun"
fn get_mongo_db_name() -> String {
    env::var("GAMAYUN_MONGO_DB_NAME").unwrap_or_else(|_| "gamayun".to_string())
}
