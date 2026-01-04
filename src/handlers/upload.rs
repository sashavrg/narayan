use crate::error::{AppError, Result};
use crate::models::{AppState, UploadResponse};
use axum::{extract::State, response::Json};
use axum::extract::Multipart;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

pub async fn handle_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    info!("Received upload request");
    let mut temp_files: Vec<(PathBuf, String)> = Vec::new(); // (temp_path, original_filename)

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Internal(format!("Failed to read multipart field: {}", e))
    })? {
        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        debug!("Processing uploaded file: {}", filename);

        // Validate FLAC extension
        if !filename.to_lowercase().ends_with(".flac") {
            return Err(AppError::InvalidFileType);
        }

        // Read file data
        debug!("Reading file data for: {}", filename);
        let data = field.bytes().await.map_err(|e| {
            let error_msg = format!("Failed to read file data for '{}': {:?}", filename, e);
            tracing::error!("{}", error_msg);
            AppError::Internal(error_msg)
        })?;
        debug!("Successfully read {} bytes for: {}", data.len(), filename);

        // Create temp file with .flac extension in the configured temp directory
        let temp_dir = state.job_manager.temp_dir();
        let mut temp_file = tempfile::Builder::new()
            .suffix(".flac")
            .tempfile_in(temp_dir)
            .map_err(|e| AppError::Io(e))?;

        // Write data to temp file (use the NamedTempFile directly)
        use std::io::Write;
        temp_file.write_all(&data).map_err(|e| {
            AppError::Io(e)
        })?;
        temp_file.flush().map_err(|e| {
            AppError::Io(e)
        })?;

        // Persist the temp file and get its path
        let temp_path = temp_file.into_temp_path().keep().map_err(|e| {
            AppError::Internal(format!("Failed to persist temp file: {}", e))
        })?;

        info!("Saved uploaded file '{}' to: {}", filename, temp_path.display());
        temp_files.push((temp_path.to_path_buf(), filename));
    }

    if temp_files.is_empty() {
        return Err(AppError::NoFilesProvided);
    }

    // Create job
    let file_count = temp_files.len();
    let job_id = state.job_manager.create_job(temp_files).await?;

    info!("Created job {} with {} files", job_id, file_count);

    Ok(Json(UploadResponse {
        job_id,
        file_count,
    }))
}
