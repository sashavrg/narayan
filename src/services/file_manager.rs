use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, info};

pub struct FileManager;

impl FileManager {
    pub async fn create_zip(files: Vec<PathBuf>, output_path: impl AsRef<Path>) -> Result<PathBuf> {
        let output = output_path.as_ref();
        info!("Creating ZIP archive: {}", output.display());

        // Use the `zip` command-line tool for simplicity
        // Collect all file paths
        let mut args = vec!["-j".to_string(), output.to_string_lossy().to_string()];

        for file_path in &files {
            if !file_path.exists() {
                debug!("Skipping non-existent file: {}", file_path.display());
                continue;
            }
            args.push(file_path.to_string_lossy().to_string());
        }

        // Run zip command
        let status = Command::new("zip")
            .args(&args)
            .status()
            .await
            .map_err(|e| AppError::ZipError(format!("Failed to run zip command: {}", e)))?;

        if !status.success() {
            return Err(AppError::ZipError(format!(
                "zip command failed with exit code: {:?}",
                status.code()
            )));
        }

        info!("ZIP archive created successfully: {}", output.display());
        Ok(output.to_path_buf())
    }

    pub async fn cleanup_directory(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if path.exists() {
            tokio::fs::remove_dir_all(path).await?;
            info!("Cleaned up directory: {}", path.display());
        }
        Ok(())
    }
}
