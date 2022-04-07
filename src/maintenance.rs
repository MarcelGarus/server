use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use filesize::PathExt;
use tokio::process::Command;

#[derive(Clone)]
pub struct Maintenance {
    this_program_started: DateTime<Utc>,
}
impl Maintenance {
    pub fn new() -> Self {
        Self {
            this_program_started: Utc::now(),
        }
    }
    pub fn server_program_uptime(&self) -> Duration {
        Utc::now().signed_duration_since(self.this_program_started)
    }
    pub async fn server_uptime(&self) -> Result<String, String> {
        let result = Command::new("uptime").output().await;
        match result {
            Ok(output) => match String::from_utf8(output.stdout) {
                Ok(output) => Ok(output.trim().to_string()),
                Err(_) => Err("The uptime command gave non-UTF8 output.".to_string()),
            },
            Err(err) => Err(format!("Failed to execute the uptime command: {:?}", err)),
        }
    }
    pub async fn log_size(&self) -> Result<u64, String> {
        let log_file = Path::new("visits.jsonl");
        match log_file.size_on_disk() {
            Ok(size) => Ok(size),
            Err(_) => Err("Couldn't get size of log file".to_string()),
        }
    }
}
