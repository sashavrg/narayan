use crate::error::{AppError, Result};
use crate::models::AppState;
use crate::services::FileManager;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::info;

pub async fn download_file(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Response> {
    info!("Download request for job: {}", job_id);

    let job = state.job_manager.get_job(&job_id)?;

    // Get the first completed file
    let file_path = job
        .result_paths
        .first()
        .ok_or_else(|| AppError::JobNotFound(format!("No completed files in job {}", job_id)))?;

    if !file_path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        )));
    }

    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download.mp3");

    let file = File::open(file_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(body)
        .unwrap())
}

pub async fn download_batch(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Response> {
    info!("Batch download request for job: {}", job_id);

    let job = state.job_manager.get_job(&job_id)?;

    if job.result_paths.is_empty() {
        return Err(AppError::JobNotFound(format!(
            "No completed files in job {}",
            job_id
        )));
    }

    // Create ZIP in temp directory
    let zip_filename = format!("{}.zip", job_id);
    let zip_path = std::env::temp_dir().join(&zip_filename);

    FileManager::create_zip(job.result_paths.clone(), &zip_path).await?;

    let file = File::open(&zip_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Schedule cleanup of ZIP file
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        let _ = tokio::fs::remove_file(zip_path).await;
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", zip_filename),
        )
        .body(body)
        .unwrap())
}
