use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub status: JobStatus,
    pub files: Vec<ConversionFile>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub result_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionFile {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub filename: String,
    pub status: FileStatus,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing {
        current_file: usize,
        total: usize,
    },
    Complete {
        #[serde(with = "duration_serde")]
        duration: Duration,
    },
    Failed {
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Pending,
    Processing,
    Complete,
    Failed(String),
}

impl Job {
    pub fn new(id: String, files: Vec<(PathBuf, String)>) -> Self {
        let now = Utc::now();
        let conversion_files: Vec<ConversionFile> = files
            .into_iter()
            .map(|(path, original_filename)| ConversionFile {
                filename: original_filename,
                input_path: path,
                output_path: None,
                status: FileStatus::Pending,
                progress: 0.0,
            })
            .collect();

        Self {
            id,
            status: JobStatus::Pending,
            files: conversion_files,
            created_at: now,
            updated_at: now,
            result_paths: Vec::new(),
        }
    }

    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn update_file_progress(&mut self, index: usize, progress: f32) {
        if let Some(file) = self.files.get_mut(index) {
            file.progress = progress;
            file.status = FileStatus::Processing;
        }
        self.updated_at = Utc::now();
    }

    pub fn complete_file(&mut self, index: usize, output_path: PathBuf) {
        if let Some(file) = self.files.get_mut(index) {
            file.output_path = Some(output_path.clone());
            file.status = FileStatus::Complete;
            file.progress = 100.0;
        }
        self.result_paths.push(output_path);
        self.updated_at = Utc::now();
    }

    pub fn fail_file(&mut self, index: usize, error: String) {
        if let Some(file) = self.files.get_mut(index) {
            file.status = FileStatus::Failed(error);
        }
        self.updated_at = Utc::now();
    }

    pub fn is_expired(&self, timeout: Duration) -> bool {
        let elapsed = Utc::now() - self.created_at;
        elapsed.to_std().unwrap_or(Duration::from_secs(0)) > timeout
    }
}

// Custom serialization for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
