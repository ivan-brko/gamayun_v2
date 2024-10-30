use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::Path;
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub enum OnDuplicateEntry {
    IgnoreNew,
    Overwrite,
    TrackChanges,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DuplicateEntryPolicy {
    pub unique_ids: Vec<String>,
    pub on_duplicate_entry: OnDuplicateEntry,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JobConfig {
    /// The name of the job, which must be unique.
    pub name: String,

    /// Path to the executable for the job.
    pub path_to_executable: String,

    /// Arguments to be given to the job.
    #[serde(default)]
    pub arguments: Vec<String>,

    /// Cron string for the job.
    pub cron_string: String,

    /// Tags for this job.
    #[serde(default)]
    pub tags: Vec<String>,

    /// How long should the system wait for results of the job before giving up.
    #[serde(default)]
    pub result_wait_timeout_millis: Option<i64>,

    /// Random trigger offset in seconds.
    #[serde(default)]
    pub random_trigger_offset_seconds: Option<i64>,

    /// Duplicate entry policy.
    #[serde(default)]
    pub duplicate_entry_policy: Option<DuplicateEntryPolicy>,
}

impl JobConfig {
    pub fn load_configs_from_directory(root_path: &str) -> Result<Vec<JobConfig>> {
        let configs = Self::recursive_load(Path::new(root_path))?;

        info!(
            "Loaded the following job configurations: {}",
            configs
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        );

        Ok(configs)
    }

    fn recursive_load(path: &Path) -> Result<Vec<JobConfig>> {
        let mut configs = Vec::new();
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    configs.extend(Self::recursive_load(&path)?);
                } else if path.is_file()
                    && path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .ends_with(".config.toml")
                {
                    let config = Self::from_file(path.to_str().unwrap())?;
                    configs.push(config);
                }
            }
        }
        Ok(configs)
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let path = Path::new(path);

        // Ensure the path exists and is a file
        if !path.is_file() {
            return Err(anyhow::anyhow!(
                "The path '{}' is not a valid file.",
                path.display()
            ));
        }

        // Read the file contents
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Get the parent directory of the file
        let parent_dir = path
            .parent()
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to get the parent directory of '{}'", path.display())
            })?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Parent directory path is not valid UTF-8"))?
            .to_string();

        // Replace the placeholder with the parent directory
        let contents = contents.replace("${CONFIGURATION_FILE_DIRECTORY}", &parent_dir);

        // Parse the TOML configuration
        let config: JobConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}
