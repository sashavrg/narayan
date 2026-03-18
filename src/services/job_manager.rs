use crate::config::Config;
use crate::error::{AppError, Result};
use crate::models::{Job, JobStatus};
use crate::services::conversion::{ConversionService, ProgressUpdate};
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::Instant;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub type JobProgressCallback = Arc<dyn Fn(String, usize, f32) + Send + Sync>;

pub struct JobManager {
    jobs: Arc<DashMap<String, Job>>,
    semaphore: Arc<Semaphore>,
    config: Config,
    progress_callback: Option<JobProgressCallback>,
}

impl JobManager {
    pub fn new(config: Config) -> Self {
        let max_concurrent = config.max_concurrent_conversions;
        Self {
            jobs: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            config,
            progress_callback: None,
        }
    }

    pub fn with_progress_callback(mut self, callback: JobProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    pub fn temp_dir(&self) -> &PathBuf {
        &self.config.temp_dir
    }

    pub fn music_library_root(&self) -> &PathBuf {
        &self.config.music_library_root
    }

    pub async fn create_job(&self, files: Vec<(PathBuf, String)>) -> Result<String> {
        if files.is_empty() {
            return Err(AppError::NoFilesProvided);
        }

        if files.len() > self.config.max_files_per_job {
            return Err(AppError::TooManyFiles(self.config.max_files_per_job));
        }

        // Validate all files are FLAC
        for (file_path, _) in &files {
            if !file_path.exists() {
                return Err(AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found: {}", file_path.display()),
                )));
            }
            if !file_path
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("flac"))
            {
                return Err(AppError::InvalidFileType);
            }
        }

        let job_id = Uuid::new_v4().to_string();
        let job = Job::new(job_id.clone(), files);

        info!("Created job {} with {} files", job_id, job.files.len());

        self.jobs.insert(job_id.clone(), job);

        // Spawn conversion task
        let manager = self.clone_for_task();
        let job_id_for_task = job_id.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.process_job(job_id_for_task.clone()).await {
                error!("Job {} failed: {}", job_id_for_task, e);
                if let Some(mut job) = manager.jobs.get_mut(&job_id_for_task) {
                    job.update_status(JobStatus::Failed {
                        error: e.to_string(),
                    });
                }
            }
        });

        Ok(job_id)
    }

    fn clone_for_task(&self) -> Self {
        Self {
            jobs: self.jobs.clone(),
            semaphore: self.semaphore.clone(),
            config: self.config.clone(),
            progress_callback: self.progress_callback.clone(),
        }
    }

    async fn process_job(&self, job_id: String) -> Result<()> {
        let start_time = Instant::now();

        // Get job details
        let files = {
            let job = self
                .jobs
                .get(&job_id)
                .ok_or_else(|| AppError::JobNotFound(job_id.clone()))?;
            job.files.clone()
        };

        let total_files = files.len();
        info!("Processing job {} with {} files", job_id, total_files);

        // Update status to processing
        if let Some(mut job) = self.jobs.get_mut(&job_id) {
            job.update_status(JobStatus::Processing {
                current_file: 0,
                total: total_files,
            });
        }

        // Create temp directory for this job
        let job_temp_dir = self.config.temp_dir.join(&job_id);
        tokio::fs::create_dir_all(&job_temp_dir).await?;

        // Process files sequentially
        for (index, file_info) in files.iter().enumerate() {
            info!(
                "Processing file {}/{}: {}",
                index + 1,
                total_files,
                file_info.filename
            );

            // Acquire semaphore permit (limits concurrent FFmpeg processes)
            let _permit =
                self.semaphore.acquire().await.map_err(|e| {
                    AppError::Internal(format!("Failed to acquire semaphore: {}", e))
                })?;

            // Update job status
            if let Some(mut job) = self.jobs.get_mut(&job_id) {
                job.update_status(JobStatus::Processing {
                    current_file: index,
                    total: total_files,
                });
            }

            // Generate output path using original filename
            let output_filename = file_info
                .filename
                .strip_suffix(".flac")
                .or_else(|| file_info.filename.strip_suffix(".FLAC"))
                .map(|s| format!("{}.mp3", s))
                .unwrap_or_else(|| format!("{}.mp3", file_info.filename));
            let output_path = job_temp_dir.join(&output_filename);

            // Create progress channel
            let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<ProgressUpdate>();

            // Spawn progress monitoring task
            let job_id_clone = job_id.clone();
            let jobs_clone = self.jobs.clone();
            let filename_clone = file_info.filename.clone();
            let callback_clone = self.progress_callback.clone();

            tokio::spawn(async move {
                while let Some(update) = progress_rx.recv().await {
                    debug!(
                        "Job {} file {}: {}%",
                        job_id_clone, filename_clone, update.percent
                    );

                    if let Some(mut job) = jobs_clone.get_mut(&job_id_clone) {
                        job.update_file_progress(index, update.percent);
                    }

                    if let Some(ref callback) = callback_clone {
                        callback(job_id_clone.clone(), index, update.percent);
                    }
                }
            });

            // Convert file
            match ConversionService::convert_flac_to_mp3(
                &file_info.input_path,
                &output_path,
                Some(progress_tx),
            )
            .await
            {
                Ok(result_path) => {
                    info!("Successfully converted: {}", result_path.display());
                    if let Some(mut job) = self.jobs.get_mut(&job_id) {
                        job.complete_file(index, result_path);
                    }
                }
                Err(e) => {
                    error!("Failed to convert {}: {}", file_info.filename, e);
                    if let Some(mut job) = self.jobs.get_mut(&job_id) {
                        job.fail_file(index, e.to_string());
                    }
                    // Continue with other files instead of failing entire job
                }
            }
        }

        // Mark job as complete
        let duration = start_time.elapsed();
        if let Some(mut job) = self.jobs.get_mut(&job_id) {
            job.update_status(JobStatus::Complete { duration });
        }

        info!("Job {} completed in {:?}", job_id, duration);

        Ok(())
    }

    pub fn get_job(&self, job_id: &str) -> Result<Job> {
        self.jobs
            .get(job_id)
            .map(|job| job.clone())
            .ok_or_else(|| AppError::JobNotFound(job_id.to_string()))
    }

    pub fn list_jobs(&self) -> Vec<Job> {
        self.jobs
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn delete_job(&self, job_id: &str) -> Result<()> {
        self.jobs
            .remove(job_id)
            .ok_or_else(|| AppError::JobNotFound(job_id.to_string()))?;

        // Clean up temp directory asynchronously
        let temp_dir = self.config.temp_dir.join(job_id);
        tokio::spawn(async move {
            if let Err(e) = tokio::fs::remove_dir_all(&temp_dir).await {
                warn!("Failed to clean up temp directory {:?}: {}", temp_dir, e);
            }
        });

        Ok(())
    }

    pub async fn cleanup_expired_jobs(&self) {
        let expired: Vec<String> = self
            .jobs
            .iter()
            .filter(|entry| entry.value().is_expired(self.config.job_timeout))
            .map(|entry| entry.key().clone())
            .collect();

        for job_id in expired {
            info!("Cleaning up expired job: {}", job_id);
            let _ = self.delete_job(&job_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_job_empty_files() {
        let config = Config::default();
        let manager = JobManager::new(config);
        let result = manager.create_job(vec![]).await;
        assert!(matches!(result, Err(AppError::NoFilesProvided)));
    }

    #[tokio::test]
    async fn test_create_job_too_many_files() {
        let mut config = Config::default();
        config.max_files_per_job = 2;
        let manager = JobManager::new(config);

        let files = vec![
            (PathBuf::from("file1.flac"), "file1.flac".to_string()),
            (PathBuf::from("file2.flac"), "file2.flac".to_string()),
            (PathBuf::from("file3.flac"), "file3.flac".to_string()),
        ];

        let result = manager.create_job(files).await;
        assert!(matches!(result, Err(AppError::TooManyFiles(2))));
    }
}
