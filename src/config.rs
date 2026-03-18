use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    /// Server bind address
    pub bind_address: String,

    /// Maximum upload size in bytes (default: 500MB)
    pub max_upload_size: usize,

    /// Maximum concurrent jobs allowed
    pub max_concurrent_jobs: usize,

    /// Maximum concurrent FFmpeg conversions
    pub max_concurrent_conversions: usize,

    /// Job timeout duration
    pub job_timeout: Duration,

    /// Maximum files per job
    pub max_files_per_job: usize,

    /// Temporary directory for conversions
    pub temp_dir: PathBuf,

    pub music_library_root: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_address: std::env::var("BIND_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            max_upload_size: std::env::var("MAX_UPLOAD_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500 * 1024 * 1024), // 500MB
            max_concurrent_jobs: std::env::var("MAX_CONCURRENT_JOBS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            max_concurrent_conversions: std::env::var("MAX_CONCURRENT_CONVERSIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
            job_timeout: Duration::from_secs(
                std::env::var("JOB_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3600), // 1 hour
            ),
            max_files_per_job: std::env::var("MAX_FILES_PER_JOB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
            temp_dir: std::env::var("TEMP_DIR")
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/tmp/narayan")),
            music_library_root: std::env::var("MUSIC_LIBRARY_ROOT")
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/mnt/storage/share/media/music")),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Self {
        // Load .env file if it exists
        let _ = dotenvy::dotenv();
        Self::default()
    }
}
