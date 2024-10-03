use dotenv::dotenv;
use mongodb::{error::Result, Client};
use std::env;

pub async fn initialize_mongo_client() -> Result<Client> {
    // Load environment variables from a .env file if present
    dotenv().ok();

    // Read the connection string from an environment variable
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI environment variable is not set");

    // Create a MongoDB client
    let client = Client::with_uri_str(&mongo_uri).await?;

    Ok(client)
}
