use crate::error::Result;
use crate::models::{AppState, JobListResponse, JobResponse};
use axum::{extract::{Path, State}, response::Json};
use tracing::debug;

pub async fn list_jobs(State(state): State<AppState>) -> Result<Json<JobListResponse>> {
    debug!("Listing all jobs");
    let jobs = state.job_manager.list_jobs();

    let job_responses: Vec<JobResponse> = jobs
        .into_iter()
        .map(|job| JobResponse {
            id: job.id.clone(),
            status: job.status,
            files: job.files,
            created_at: job.created_at.to_rfc3339(),
            updated_at: job.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(JobListResponse {
        jobs: job_responses,
    }))
}

pub async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<JobResponse>> {
    debug!("Getting job: {}", job_id);
    let job = state.job_manager.get_job(&job_id)?;

    Ok(Json(JobResponse {
        id: job.id.clone(),
        status: job.status,
        files: job.files,
        created_at: job.created_at.to_rfc3339(),
        updated_at: job.updated_at.to_rfc3339(),
    }))
}
