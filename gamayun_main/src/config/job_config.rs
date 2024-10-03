use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub enum OnDuplicateEntry {
    IgnoreNew,
    Overwrite,
    TrackChanges,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DuplicateEntryPolicy {
    unique_ids: Vec<String>,
    on_duplicate_entry: OnDuplicateEntry,
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
    pub result_wait_timeout_millis: Option<u64>,

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
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Replace ${CONFIGURATION_FILE_PATH} with the actual path
        let contents = contents.replace("${CONFIGURATION_FILE_PATH}", path);

        let config: JobConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}
