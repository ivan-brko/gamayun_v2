use anyhow::{Context, Result};
use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SendGridConfig {
    pub api_key: String,
    pub from_email: String,
    pub to_emails: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub sendgrid_config: Option<SendGridConfig>,
}

pub fn initialize_app_config(config_root: String) -> Result<AppConfig> {
    let settings = Config::builder()
        // Add in `./app_config.toml`
        .add_source(config::File::with_name(&format!("{}/app_config", config_root)).required(false))
        .add_source(
            config::Environment::with_prefix("GAMAYUN")
                .prefix_separator("___")
                .separator("__")
                .list_separator(" "),
        )
        .build()
        .context("Error while initializing application configuration")?;

    let deserialized_config: AppConfig = settings.try_deserialize()?;
    Ok(deserialized_config)
}
